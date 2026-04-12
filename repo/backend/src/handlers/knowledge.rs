//! Knowledge module HTTP handlers — categories, knowledge points, questions.
//! Every mutation here calls `AuditService::log` before returning so the
//! audit_logs table records actor + before/after hashes.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use shared::AuditAction;

use crate::error::AppResult;
use crate::middleware::require_role::{AuthenticatedUser, RequireCurator};
use crate::middleware::session::CurrentUser;
use crate::models::knowledge::{Category, KnowledgePoint, Question};
use crate::services::audit_service::AuditService;
use crate::services::knowledge_service::{
    BulkUpdate, CategoryNode, ConflictPreview, CreateCategoryInput, CreateKnowledgePointInput,
    FilterParams, KnowledgeService, ReferenceCount, UpdateCategoryInput, UpdateKnowledgePointInput,
};
use crate::services::question_service::{
    CreateQuestionInput, QuestionFilter, QuestionService, UpdateQuestionInput,
};
use crate::AppState;

// ─── Categories ─────────────────────────────────────────────────────────

pub async fn list_categories(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
) -> AppResult<Json<Vec<Category>>> {
    let svc = KnowledgeService::new(state.db.clone());
    Ok(Json(svc.list_categories().await?))
}

pub async fn get_tree(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
) -> AppResult<Json<Vec<CategoryNode>>> {
    let svc = KnowledgeService::new(state.db.clone());
    Ok(Json(svc.get_tree().await?))
}

pub async fn create_category(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Json(input): Json<CreateCategoryInput>,
) -> AppResult<Json<Category>> {
    let svc = KnowledgeService::new(state.db.clone());
    let cat = svc.create_category(input, &user.id).await?;
    let audit = AuditService::new(state.db.clone());
    let after_hash = AuditService::compute_hash(&serde_json::to_string(&cat)?);
    audit
        .log(
            &user.id,
            AuditAction::Create,
            "category",
            Some(&cat.id),
            None,
            Some(after_hash),
            None,
        )
        .await?;
    Ok(Json(cat))
}

pub async fn update_category(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Path(id): Path<String>,
    Json(input): Json<UpdateCategoryInput>,
) -> AppResult<Json<Category>> {
    let svc = KnowledgeService::new(state.db.clone());
    let before = svc.get_category(&id).await?;
    let before_hash = AuditService::compute_hash(&serde_json::to_string(&before)?);
    let updated = svc.update_category(&id, input).await?;
    let after_hash = AuditService::compute_hash(&serde_json::to_string(&updated)?);
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "category",
            Some(&id),
            Some(before_hash),
            Some(after_hash),
            None,
        )
        .await?;
    Ok(Json(updated))
}

pub async fn delete_category(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = KnowledgeService::new(state.db.clone());
    let before = svc.get_category(&id).await?;
    let before_hash = AuditService::compute_hash(&serde_json::to_string(&before)?);
    svc.delete_category(&id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Delete,
            "category",
            Some(&id),
            Some(before_hash),
            None,
            None,
        )
        .await?;
    Ok(Json(json!({"success": true})))
}

pub async fn category_reference_count(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
    Path(id): Path<String>,
) -> AppResult<Json<ReferenceCount>> {
    let svc = KnowledgeService::new(state.db.clone());
    Ok(Json(svc.get_reference_count(&id).await?))
}

#[derive(Deserialize)]
pub struct MergeRequest {
    pub source_id: String,
    pub target_id: String,
}

pub async fn merge_categories(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Json(req): Json<MergeRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = KnowledgeService::new(state.db.clone());
    svc.merge_nodes(&req.source_id, &req.target_id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::MergeNodes,
            "category",
            Some(&req.source_id),
            None,
            Some(AuditService::compute_hash(&req.target_id)),
            None,
        )
        .await?;
    Ok(Json(json!({"success": true, "merged_into": req.target_id})))
}

// ─── Knowledge Points ───────────────────────────────────────────────────

pub async fn create_knowledge_point(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Json(input): Json<CreateKnowledgePointInput>,
) -> AppResult<Json<KnowledgePoint>> {
    let svc = KnowledgeService::new(state.db.clone());
    let kp = svc.create_knowledge_point(input, &user.id).await?;
    let after = AuditService::compute_hash(&serde_json::to_string(&kp)?);
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Create,
            "knowledge_point",
            Some(&kp.id),
            None,
            Some(after),
            None,
        )
        .await?;
    Ok(Json(kp))
}

pub async fn update_knowledge_point(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Path(id): Path<String>,
    Json(input): Json<UpdateKnowledgePointInput>,
) -> AppResult<Json<KnowledgePoint>> {
    let svc = KnowledgeService::new(state.db.clone());
    let before = svc.get_knowledge_point(&id).await?;
    let before_hash = AuditService::compute_hash(&serde_json::to_string(&before)?);
    let updated = svc.update_knowledge_point(&id, input).await?;
    let after_hash = AuditService::compute_hash(&serde_json::to_string(&updated)?);
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "knowledge_point",
            Some(&id),
            Some(before_hash),
            Some(after_hash),
            None,
        )
        .await?;
    Ok(Json(updated))
}

pub async fn delete_knowledge_point(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = KnowledgeService::new(state.db.clone());
    let before = svc.get_knowledge_point(&id).await?;
    let before_hash = AuditService::compute_hash(&serde_json::to_string(&before)?);
    svc.delete_knowledge_point(&id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Delete,
            "knowledge_point",
            Some(&id),
            Some(before_hash),
            None,
            None,
        )
        .await?;
    Ok(Json(json!({"success": true})))
}

#[derive(Deserialize, Default)]
pub struct FilterQuery {
    pub category_id: Option<String>,
    pub difficulty_min: Option<i64>,
    pub difficulty_max: Option<i64>,
    pub discrimination_min: Option<f64>,
    pub discrimination_max: Option<f64>,
    /// Legacy single-tag parameter — retained for backwards compatibility.
    pub tag: Option<String>,
    /// Preferred multi-tag parameter: comma-separated list (`tags=foo,bar`).
    /// Serde_urlencoded can't deserialize repeated `tags=` into `Vec`, so we
    /// accept the CSV form and split it in the handler.
    pub tags: Option<String>,
    pub chapter: Option<String>,
}

pub async fn list_knowledge_points(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(q): Query<FilterQuery>,
) -> AppResult<Json<Vec<KnowledgePoint>>> {
    let actor = user.id.clone();
    // Anti-abuse: reject if the caller is currently in exponential backoff
    // from repeated invalid searches. `check` is a fast DashMap read.
    state.invalid_search_tracker.check(&actor)?;
    let svc = KnowledgeService::new(state.db.clone());
    let mut tags: Vec<String> = Vec::new();
    if let Some(t) = q.tag {
        if !t.is_empty() {
            tags.push(t);
        }
    }
    if let Some(csv) = q.tags {
        for piece in csv.split(',') {
            let p = piece.trim();
            if !p.is_empty() {
                tags.push(p.to_string());
            }
        }
    }
    // De-duplicate while preserving order so repeat params don't blow up the
    // generated LIKE clause.
    let mut seen = std::collections::HashSet::new();
    tags.retain(|t| seen.insert(t.clone()));
    let filter = FilterParams {
        category_id: q.category_id,
        tags,
        difficulty_min: q.difficulty_min,
        difficulty_max: q.difficulty_max,
        discrimination_min: q.discrimination_min,
        discrimination_max: q.discrimination_max,
        chapter: q.chapter,
    };
    let has_criteria = filter.category_id.is_some()
        || !filter.tags.is_empty()
        || filter.difficulty_min.is_some()
        || filter.difficulty_max.is_some()
        || filter.discrimination_min.is_some()
        || filter.discrimination_max.is_some()
        || filter.chapter.is_some();
    let rows = svc.filter_knowledge_points(&filter).await?;
    if has_criteria {
        if rows.is_empty() {
            state.invalid_search_tracker.record_invalid(&actor);
        } else {
            state.invalid_search_tracker.reset(&actor);
        }
    }
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct BulkUpdateRequest {
    pub ids: Vec<String>,
    pub changes: BulkUpdate,
}

pub async fn bulk_preview(
    State(state): State<AppState>,
    RequireCurator(_user): RequireCurator,
    Json(req): Json<BulkUpdateRequest>,
) -> AppResult<Json<Vec<ConflictPreview>>> {
    let svc = KnowledgeService::new(state.db.clone());
    Ok(Json(svc.preview_bulk_conflicts(&req.ids, &req.changes).await?))
}

pub async fn bulk_apply(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Json(req): Json<BulkUpdateRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = KnowledgeService::new(state.db.clone());
    let count = svc.bulk_update(&req.ids, &req.changes).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::BulkUpdate,
            "knowledge_point",
            None,
            None,
            Some(AuditService::compute_hash(&format!("ids={} count={}", req.ids.len(), count))),
            None,
        )
        .await?;
    Ok(Json(json!({"updated": count})))
}

// ─── Questions ──────────────────────────────────────────────────────────

pub async fn list_questions(
    State(state): State<AppState>,
    AuthenticatedUser(_): AuthenticatedUser,
    Query(filter): Query<QuestionFilter>,
) -> AppResult<Json<Vec<Question>>> {
    let svc = QuestionService::new(state.db.clone());
    Ok(Json(svc.filter(&filter).await?))
}

pub async fn create_question(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Json(input): Json<CreateQuestionInput>,
) -> AppResult<Json<Question>> {
    let svc = QuestionService::new(state.db.clone());
    let q = svc.create(input, &user.id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Create,
            "question",
            Some(&q.id),
            None,
            Some(AuditService::compute_hash(&serde_json::to_string(&q)?)),
            None,
        )
        .await?;
    Ok(Json(q))
}

pub async fn update_question(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Path(id): Path<String>,
    Json(input): Json<UpdateQuestionInput>,
) -> AppResult<Json<Question>> {
    let svc = QuestionService::new(state.db.clone());
    let before = svc.get(&id).await?;
    let updated = svc.update(&id, input).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "question",
            Some(&id),
            Some(AuditService::compute_hash(&serde_json::to_string(&before)?)),
            Some(AuditService::compute_hash(&serde_json::to_string(&updated)?)),
            None,
        )
        .await?;
    Ok(Json(updated))
}

pub async fn delete_question(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = QuestionService::new(state.db.clone());
    svc.delete(&id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Delete,
            "question",
            Some(&id),
            None,
            None,
            None,
        )
        .await?;
    Ok(Json(json!({"success": true})))
}

#[derive(Deserialize)]
pub struct LinkRequest {
    pub knowledge_point_id: String,
}

pub async fn link_question(
    State(state): State<AppState>,
    RequireCurator(user): RequireCurator,
    Path(id): Path<String>,
    Json(req): Json<LinkRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = QuestionService::new(state.db.clone());
    svc.link_to_kp(&id, &req.knowledge_point_id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "question_link",
            Some(&id),
            None,
            Some(AuditService::compute_hash(&req.knowledge_point_id)),
            None,
        )
        .await?;
    Ok(Json(json!({"success": true})))
}

// Suppress an unused import warning when compiling without later phases.
#[allow(dead_code)]
fn _force_user_import(_: CurrentUser) {}
