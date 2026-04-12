//! Knowledge service.
//!
//! Owns reads/writes against `categories`, `knowledge_points` and the
//! `knowledge_question_links` join table. The two business rules from SPEC.md
//! that live in this module:
//!
//! 1. **DAG cycle detection** — `check_would_create_cycle` walks descendants of
//!    the proposed child and refuses any edge that would loop back to the parent.
//! 2. **1000-record bulk edit cap** — `bulk_update` rejects requests larger than
//!    1000 ids with a `Validation` error before touching the database.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::knowledge::{Category, KnowledgePoint};

pub const MAX_BULK_EDIT: usize = 1000;

#[derive(Clone)]
pub struct KnowledgeService {
    pub db: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategoryInput {
    pub name: String,
    pub parent_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategoryInput {
    pub name: Option<String>,
    pub parent_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKnowledgePointInput {
    pub category_id: String,
    pub title: String,
    pub content: String,
    pub difficulty: i64,
    pub discrimination: f64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateKnowledgePointInput {
    pub category_id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub difficulty: Option<i64>,
    pub discrimination: Option<f64>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BulkUpdate {
    pub category_id: Option<String>,
    pub difficulty: Option<i64>,
    pub discrimination: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPreview {
    pub kp_id: String,
    pub field: String,
    pub current_value: String,
    pub proposed_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterParams {
    pub category_id: Option<String>,
    pub tags: Vec<String>,
    pub difficulty_min: Option<i64>,
    pub difficulty_max: Option<i64>,
    pub discrimination_min: Option<f64>,
    pub discrimination_max: Option<f64>,
    pub chapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceCount {
    pub direct_kp_count: i64,
    pub child_category_count: i64,
    pub indirect_question_count: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryNode {
    pub category: Category,
    pub children: Vec<CategoryNode>,
    pub kp_count: i64,
}

impl KnowledgeService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Categories
    // ─────────────────────────────────────────────────────────────────────

    pub async fn list_categories(&self) -> AppResult<Vec<Category>> {
        let rows = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories WHERE deleted_at IS NULL ORDER BY level, name",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    pub async fn get_category(&self, id: &str) -> AppResult<Category> {
        let row = sqlx::query_as::<_, Category>(
            "SELECT * FROM categories WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;
        row.ok_or(AppError::NotFound)
    }

    pub async fn create_category(
        &self,
        input: CreateCategoryInput,
        actor_id: &str,
    ) -> AppResult<Category> {
        if input.name.trim().is_empty() {
            return Err(AppError::Validation("category name is required".into()));
        }
        // Compute level: parent.level + 1, or 0 for root
        let level = if let Some(parent_id) = &input.parent_id {
            let parent = self.get_category(parent_id).await?;
            parent.level + 1
        } else {
            0
        };
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO categories (id, name, parent_id, level, description, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.parent_id)
        .bind(level)
        .bind(&input.description)
        .bind(actor_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;
        self.get_category(&id).await
    }

    pub async fn update_category(
        &self,
        id: &str,
        input: UpdateCategoryInput,
    ) -> AppResult<Category> {
        let current = self.get_category(id).await?;
        // Validate proposed parent change does not create a cycle.
        if let Some(new_parent) = &input.parent_id {
            if new_parent == id {
                return Err(AppError::Conflict("category cannot be its own parent".into()));
            }
            if self.check_would_create_cycle(new_parent, id).await? {
                return Err(AppError::Conflict(
                    "moving this category would create a cycle in the tree".into(),
                ));
            }
        }
        let new_name = input.name.unwrap_or(current.name);
        let new_parent = input.parent_id.or(current.parent_id);
        let new_desc = input.description.or(current.description);
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE categories SET name = ?, parent_id = ?, description = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&new_name)
        .bind(&new_parent)
        .bind(&new_desc)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;
        self.get_category(id).await
    }

    pub async fn delete_category(&self, id: &str) -> AppResult<()> {
        let refs = self.get_reference_count(id).await?;
        if refs.total > 0 {
            return Err(AppError::Conflict(format!(
                "cannot delete: {} references still attached (use merge instead)",
                refs.total
            )));
        }
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE categories SET deleted_at = ? WHERE id = ?")
            .bind(&now)
            .bind(id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Build the full category tree as a nested `CategoryNode` structure.
    pub async fn get_tree(&self) -> AppResult<Vec<CategoryNode>> {
        let cats = self.list_categories().await?;
        // Index children by parent_id
        use std::collections::HashMap;
        let mut children_by_parent: HashMap<Option<String>, Vec<Category>> = HashMap::new();
        for c in cats {
            children_by_parent
                .entry(c.parent_id.clone())
                .or_default()
                .push(c);
        }
        // Recursive builder
        fn build(
            parent_id: Option<String>,
            map: &std::collections::HashMap<Option<String>, Vec<Category>>,
            kp_counts: &std::collections::HashMap<String, i64>,
        ) -> Vec<CategoryNode> {
            let mut out = Vec::new();
            if let Some(children) = map.get(&parent_id) {
                for c in children {
                    let id = c.id.clone();
                    let kp_count = kp_counts.get(&id).copied().unwrap_or(0);
                    out.push(CategoryNode {
                        category: c.clone(),
                        children: build(Some(id), map, kp_counts),
                        kp_count,
                    });
                }
            }
            out
        }
        // Pre-fetch kp counts per category
        let counts_rows: Vec<(String, i64)> =
            sqlx::query_as("SELECT category_id, COUNT(*) FROM knowledge_points GROUP BY category_id")
                .fetch_all(&self.db)
                .await?;
        let kp_counts: std::collections::HashMap<String, i64> = counts_rows.into_iter().collect();
        Ok(build(None, &children_by_parent, &kp_counts))
    }

    /// DFS-based cycle detection for the category DAG.
    /// Returns `true` if adding the edge `parent_id → child_id` would create
    /// a cycle (i.e. `parent_id` is reachable from `child_id` via existing
    /// parent→child edges).
    pub async fn check_would_create_cycle(
        &self,
        parent_id: &str,
        child_id: &str,
    ) -> AppResult<bool> {
        if parent_id == child_id {
            return Ok(true);
        }
        let mut visited: HashSet<String> = HashSet::new();
        let mut stack: Vec<String> = vec![child_id.to_string()];
        while let Some(node) = stack.pop() {
            if node == parent_id {
                return Ok(true); // would create a cycle
            }
            if !visited.insert(node.clone()) {
                continue;
            }
            let children: Vec<String> = sqlx::query_scalar(
                "SELECT id FROM categories WHERE parent_id = ? AND deleted_at IS NULL",
            )
            .bind(&node)
            .fetch_all(&self.db)
            .await?;
            stack.extend(children);
        }
        Ok(false)
    }

    /// Reference count for a category — used by the merge UI to show what would
    /// be moved. Returns direct knowledge_points, immediate child categories,
    /// and the count of questions linked to those knowledge points.
    pub async fn get_reference_count(&self, id: &str) -> AppResult<ReferenceCount> {
        let direct_kp_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_points WHERE category_id = ?")
                .bind(id)
                .fetch_one(&self.db)
                .await?;
        let child_category_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM categories WHERE parent_id = ? AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;
        let indirect_question_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM questions WHERE knowledge_point_id IN
             (SELECT id FROM knowledge_points WHERE category_id = ?)",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;
        let total = direct_kp_count + child_category_count + indirect_question_count;
        Ok(ReferenceCount {
            direct_kp_count,
            child_category_count,
            indirect_question_count,
            total,
        })
    }

    /// Merge `source_id` into `target_id`: re-parent every direct child category
    /// and every knowledge point under source so they hang off target instead,
    /// then soft-delete source. Refuses the operation when:
    ///
    ///   • either node is missing
    ///   • the merge would create a cycle (target is in source's subtree)
    ///   • source equals target
    pub async fn merge_nodes(&self, source_id: &str, target_id: &str) -> AppResult<()> {
        if source_id == target_id {
            return Err(AppError::Validation(
                "cannot merge a category into itself".into(),
            ));
        }
        let _source = self.get_category(source_id).await?;
        let _target = self.get_category(target_id).await?;

        // Cycle check: target must not be a descendant of source.
        let mut visited: HashSet<String> = HashSet::new();
        let mut stack: Vec<String> = vec![source_id.to_string()];
        while let Some(node) = stack.pop() {
            if node == target_id {
                return Err(AppError::Conflict(
                    "merge would create a cycle in the category tree".into(),
                ));
            }
            if !visited.insert(node.clone()) {
                continue;
            }
            let children: Vec<String> = sqlx::query_scalar(
                "SELECT id FROM categories WHERE parent_id = ? AND deleted_at IS NULL",
            )
            .bind(&node)
            .fetch_all(&self.db)
            .await?;
            stack.extend(children);
        }

        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.begin().await?;
        sqlx::query(
            "UPDATE categories SET parent_id = ?, updated_at = ? WHERE parent_id = ? AND deleted_at IS NULL",
        )
        .bind(target_id)
        .bind(&now)
        .bind(source_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE knowledge_points SET category_id = ?, updated_at = ? WHERE category_id = ?",
        )
        .bind(target_id)
        .bind(&now)
        .bind(source_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE categories SET deleted_at = ? WHERE id = ?")
            .bind(&now)
            .bind(source_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────
    // Knowledge points
    // ─────────────────────────────────────────────────────────────────────

    pub async fn create_knowledge_point(
        &self,
        input: CreateKnowledgePointInput,
        actor_id: &str,
    ) -> AppResult<KnowledgePoint> {
        if input.title.trim().is_empty() {
            return Err(AppError::Validation("title is required".into()));
        }
        if !(1..=5).contains(&input.difficulty) {
            return Err(AppError::Validation("difficulty must be 1..=5".into()));
        }
        if !(-1.0..=1.0).contains(&input.discrimination) {
            return Err(AppError::Validation(
                "discrimination must be in [-1.0, 1.0]".into(),
            ));
        }
        // Confirm category exists.
        let _ = self.get_category(&input.category_id).await?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let tags_json = serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".to_string());
        sqlx::query(
            "INSERT INTO knowledge_points (id, category_id, title, content, difficulty, discrimination, tags, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&input.category_id)
        .bind(&input.title)
        .bind(&input.content)
        .bind(input.difficulty)
        .bind(input.discrimination)
        .bind(&tags_json)
        .bind(actor_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;
        self.get_knowledge_point(&id).await
    }

    pub async fn get_knowledge_point(&self, id: &str) -> AppResult<KnowledgePoint> {
        let row = sqlx::query_as::<_, KnowledgePoint>(
            "SELECT * FROM knowledge_points WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;
        row.ok_or(AppError::NotFound)
    }

    pub async fn update_knowledge_point(
        &self,
        id: &str,
        input: UpdateKnowledgePointInput,
    ) -> AppResult<KnowledgePoint> {
        let current = self.get_knowledge_point(id).await?;
        let new_category = input.category_id.unwrap_or(current.category_id);
        let new_title = input.title.unwrap_or(current.title);
        let new_content = input.content.unwrap_or(current.content);
        let new_diff = input.difficulty.unwrap_or(current.difficulty);
        let new_disc = input.discrimination.unwrap_or(current.discrimination);
        let new_tags = match input.tags {
            Some(t) => serde_json::to_string(&t).unwrap_or_else(|_| "[]".to_string()),
            None => current.tags,
        };
        if !(1..=5).contains(&new_diff) {
            return Err(AppError::Validation("difficulty must be 1..=5".into()));
        }
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE knowledge_points
             SET category_id = ?, title = ?, content = ?, difficulty = ?, discrimination = ?, tags = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&new_category)
        .bind(&new_title)
        .bind(&new_content)
        .bind(new_diff)
        .bind(new_disc)
        .bind(&new_tags)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;
        self.get_knowledge_point(id).await
    }

    pub async fn delete_knowledge_point(&self, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM knowledge_points WHERE id = ?")
            .bind(id)
            .execute(&self.db)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    /// Bulk-edit knowledge points. Hard-caps the request at 1000 ids and refuses
    /// anything larger with a `Validation` error before the SQL runs.
    pub async fn bulk_update(
        &self,
        ids: &[String],
        changes: &BulkUpdate,
    ) -> AppResult<usize> {
        if ids.len() > MAX_BULK_EDIT {
            return Err(AppError::Validation(format!(
                "bulk edit limited to {MAX_BULK_EDIT} records, got {}",
                ids.len()
            )));
        }
        if ids.is_empty() {
            return Ok(0);
        }
        // Deduplicate incoming ids so the same record is only counted once.
        let mut seen = std::collections::HashSet::new();
        let deduped: Vec<&String> = ids.iter().filter(|id| seen.insert(id.as_str())).collect();
        let now = Utc::now().to_rfc3339();
        let mut tx = self.db.begin().await?;
        // Track which *unique record ids* were actually changed (not statement
        // row-counts, which can overcount when multiple fields update the same row).
        let mut touched_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        for chunk in deduped.chunks(200) {
            let placeholders = vec!["?"; chunk.len()].join(",");
            let chunk_ids: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();

            // Prefetch which IDs in this chunk actually exist in the DB so we
            // only count truly updated rows, not the full chunk.
            let exist_q = format!(
                "SELECT id FROM knowledge_points WHERE id IN ({})",
                placeholders
            );
            let mut exist_query = sqlx::query_scalar::<_, String>(&exist_q);
            for id in &chunk_ids {
                exist_query = exist_query.bind(*id);
            }
            let existing: std::collections::HashSet<String> =
                exist_query.fetch_all(&mut *tx).await?.into_iter().collect();

            if let Some(category_id) = &changes.category_id {
                let q = format!(
                    "UPDATE knowledge_points SET category_id = ?, updated_at = ? WHERE id IN ({})",
                    placeholders
                );
                let mut query = sqlx::query(&q).bind(category_id).bind(&now);
                for id in &chunk_ids {
                    query = query.bind(*id);
                }
                let res = query.execute(&mut *tx).await?;
                if res.rows_affected() > 0 {
                    touched_ids.extend(existing.iter().cloned());
                }
            }
            if let Some(diff) = changes.difficulty {
                if !(1..=5).contains(&diff) {
                    return Err(AppError::Validation("difficulty must be 1..=5".into()));
                }
                let q = format!(
                    "UPDATE knowledge_points SET difficulty = ?, updated_at = ? WHERE id IN ({})",
                    placeholders
                );
                let mut query = sqlx::query(&q).bind(diff).bind(&now);
                for id in &chunk_ids {
                    query = query.bind(*id);
                }
                let res = query.execute(&mut *tx).await?;
                if res.rows_affected() > 0 {
                    touched_ids.extend(existing.iter().cloned());
                }
            }
            if let Some(disc) = changes.discrimination {
                let q = format!(
                    "UPDATE knowledge_points SET discrimination = ?, updated_at = ? WHERE id IN ({})",
                    placeholders
                );
                let mut query = sqlx::query(&q).bind(disc).bind(&now);
                for id in &chunk_ids {
                    query = query.bind(*id);
                }
                let res = query.execute(&mut *tx).await?;
                if res.rows_affected() > 0 {
                    touched_ids.extend(existing.iter().cloned());
                }
            }
        }
        tx.commit().await?;
        Ok(touched_ids.len())
    }

    /// Preview which rows would change under a bulk edit. Returns one
    /// `ConflictPreview` per (row, field) where the current value differs from
    /// the proposed value.
    pub async fn preview_bulk_conflicts(
        &self,
        ids: &[String],
        changes: &BulkUpdate,
    ) -> AppResult<Vec<ConflictPreview>> {
        if ids.len() > MAX_BULK_EDIT {
            return Err(AppError::Validation(format!(
                "bulk preview limited to {MAX_BULK_EDIT} records",
            )));
        }
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = vec!["?"; ids.len()].join(",");
        let q = format!(
            "SELECT id, category_id, difficulty, discrimination FROM knowledge_points WHERE id IN ({})",
            placeholders
        );
        let mut query = sqlx::query_as::<_, (String, String, i64, f64)>(&q);
        for id in ids {
            query = query.bind(id);
        }
        let rows = query.fetch_all(&self.db).await?;

        let mut conflicts = Vec::new();
        for (kp_id, cur_cat, cur_diff, cur_disc) in rows {
            if let Some(target_cat) = &changes.category_id {
                if &cur_cat != target_cat {
                    conflicts.push(ConflictPreview {
                        kp_id: kp_id.clone(),
                        field: "category_id".into(),
                        current_value: cur_cat.clone(),
                        proposed_value: target_cat.clone(),
                    });
                }
            }
            if let Some(target_diff) = changes.difficulty {
                if cur_diff != target_diff {
                    conflicts.push(ConflictPreview {
                        kp_id: kp_id.clone(),
                        field: "difficulty".into(),
                        current_value: cur_diff.to_string(),
                        proposed_value: target_diff.to_string(),
                    });
                }
            }
            if let Some(target_disc) = changes.discrimination {
                if (cur_disc - target_disc).abs() > 1e-9 {
                    conflicts.push(ConflictPreview {
                        kp_id,
                        field: "discrimination".into(),
                        current_value: format!("{:.3}", cur_disc),
                        proposed_value: format!("{:.3}", target_disc),
                    });
                }
            }
        }
        Ok(conflicts)
    }

    /// Combined-filter query — chapter, tags, difficulty range, discrimination
    /// range, category id. Returns at most 500 results so the UI table never
    /// blows up on bad filters.
    pub async fn filter_knowledge_points(
        &self,
        filter: &FilterParams,
    ) -> AppResult<Vec<KnowledgePoint>> {
        let mut q = String::from("SELECT * FROM knowledge_points WHERE 1=1");
        let mut binds: Vec<String> = Vec::new();
        if let Some(cat) = &filter.category_id {
            q.push_str(" AND category_id = ?");
            binds.push(cat.clone());
        }
        if let Some(min) = filter.difficulty_min {
            q.push_str(" AND difficulty >= ?");
            binds.push(min.to_string());
        }
        if let Some(max) = filter.difficulty_max {
            q.push_str(" AND difficulty <= ?");
            binds.push(max.to_string());
        }
        if let Some(min) = filter.discrimination_min {
            q.push_str(" AND discrimination >= ?");
            binds.push(min.to_string());
        }
        if let Some(max) = filter.discrimination_max {
            q.push_str(" AND discrimination <= ?");
            binds.push(max.to_string());
        }
        for tag in &filter.tags {
            q.push_str(" AND tags LIKE ?");
            binds.push(format!("%\"{tag}\"%"));
        }
        // Chapter is stored on `questions`; restrict KPs to those linked to at
        // least one question in that chapter (either via direct
        // `knowledge_point_id` FK or via the `knowledge_question_links` table).
        if let Some(chapter) = &filter.chapter {
            if !chapter.is_empty() {
                q.push_str(
                    " AND id IN ( \
                        SELECT knowledge_point_id FROM questions \
                          WHERE chapter = ? AND knowledge_point_id IS NOT NULL \
                        UNION \
                        SELECT l.knowledge_point_id FROM knowledge_question_links l \
                          JOIN questions q2 ON q2.id = l.question_id \
                          WHERE q2.chapter = ? \
                    )",
                );
                binds.push(chapter.clone());
                binds.push(chapter.clone());
            }
        }
        q.push_str(" ORDER BY updated_at DESC LIMIT 500");

        let mut query = sqlx::query_as::<_, KnowledgePoint>(&q);
        for b in &binds {
            query = query.bind(b);
        }
        let rows = query.fetch_all(&self.db).await?;
        Ok(rows)
    }
}
