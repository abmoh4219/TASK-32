//! Outcome / IP registration service.
//!
//! Two business rules from SPEC.md live here:
//!
//! 1. **Contribution shares must total exactly 100%** at submission time.
//!    Enforced in `submit_outcome`. `add_contributor` additionally rejects any
//!    addition that would push the running total over 100.
//! 2. **Duplicate detection** via Jaro-Winkler similarity (≥0.85 on title,
//!    ≥0.80 on the first 200 chars of the abstract, plus exact certificate
//!    number match) — implemented in `find_duplicates` and surfaced from
//!    `create_outcome` as a non-blocking candidate list.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::outcome::{Outcome, OutcomeContributor};

pub const TITLE_SIMILARITY_THRESHOLD: f64 = 0.85;
pub const ABSTRACT_SIMILARITY_THRESHOLD: f64 = 0.80;
pub const ABSTRACT_COMPARE_LEN: usize = 200;

#[derive(Clone)]
pub struct OutcomeService {
    pub db: SqlitePool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOutcomeInput {
    pub r#type: String,
    pub title: String,
    pub abstract_snippet: String,
    pub certificate_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCandidate {
    pub id: String,
    pub title: String,
    pub similarity_score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOutcomeResult {
    pub outcome: Outcome,
    pub duplicate_candidates: Vec<DuplicateCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddContributorInput {
    pub user_id: String,
    pub share_percentage: i64,
    pub role_in_work: Option<String>,
}

impl OutcomeService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Create an outcome and run duplicate detection. The outcome is always
    /// inserted; duplicate matches are returned as candidates so the UI can
    /// surface a "review similar items" warning before the user proceeds.
    pub async fn create_outcome(
        &self,
        input: CreateOutcomeInput,
        actor_id: &str,
    ) -> AppResult<CreateOutcomeResult> {
        if input.title.trim().is_empty() {
            return Err(AppError::Validation("outcome title is required".into()));
        }
        if !["paper", "patent", "competition_result", "software_copyright"]
            .contains(&input.r#type.as_str())
        {
            return Err(AppError::Validation(format!(
                "unknown outcome type: {}",
                input.r#type
            )));
        }

        let candidates = self
            .find_duplicates(
                &input.title,
                &input.abstract_snippet,
                input.certificate_number.as_deref(),
            )
            .await?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO outcomes (id, type, title, abstract_snippet, certificate_number, status, created_by, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 'draft', ?, ?, ?)",
        )
        .bind(&id)
        .bind(&input.r#type)
        .bind(&input.title)
        .bind(&input.abstract_snippet)
        .bind(&input.certificate_number)
        .bind(actor_id)
        .bind(&now)
        .bind(&now)
        .execute(&self.db)
        .await?;

        let outcome = self.get_outcome(&id).await?;
        Ok(CreateOutcomeResult {
            outcome,
            duplicate_candidates: candidates,
        })
    }

    pub async fn get_outcome(&self, id: &str) -> AppResult<Outcome> {
        let row = sqlx::query_as::<_, Outcome>("SELECT * FROM outcomes WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db)
            .await?;
        row.ok_or(AppError::NotFound)
    }

    pub async fn list_outcomes(&self, limit: i64) -> AppResult<Vec<Outcome>> {
        let rows = sqlx::query_as::<_, Outcome>(
            "SELECT * FROM outcomes ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    /// Jaro-Winkler / exact-match scan against the existing outcomes table.
    /// Anything that crosses the title or abstract threshold — or matches the
    /// certificate number exactly — is returned as a `DuplicateCandidate`.
    pub async fn find_duplicates(
        &self,
        title: &str,
        abstract_snippet: &str,
        certificate_number: Option<&str>,
    ) -> AppResult<Vec<DuplicateCandidate>> {
        let existing = sqlx::query_as::<_, Outcome>(
            "SELECT * FROM outcomes ORDER BY created_at DESC LIMIT 1000",
        )
        .fetch_all(&self.db)
        .await?;

        let mut out = Vec::new();
        let abstract_input = head(abstract_snippet, ABSTRACT_COMPARE_LEN);
        for cand in existing {
            // Exact certificate number match
            if let (Some(input_cert), Some(existing_cert)) =
                (certificate_number, cand.certificate_number.as_deref())
            {
                if !input_cert.is_empty() && input_cert == existing_cert {
                    out.push(DuplicateCandidate {
                        id: cand.id.clone(),
                        title: cand.title.clone(),
                        similarity_score: 1.0,
                        reason: "certificate number matches".into(),
                    });
                    continue;
                }
            }
            let title_score = strsim::jaro_winkler(title, &cand.title);
            if title_score >= TITLE_SIMILARITY_THRESHOLD {
                out.push(DuplicateCandidate {
                    id: cand.id.clone(),
                    title: cand.title.clone(),
                    similarity_score: title_score,
                    reason: format!("title similarity {:.2}", title_score),
                });
                continue;
            }
            let abstract_score =
                strsim::jaro_winkler(abstract_input, head(&cand.abstract_snippet, ABSTRACT_COMPARE_LEN));
            if abstract_score >= ABSTRACT_SIMILARITY_THRESHOLD {
                out.push(DuplicateCandidate {
                    id: cand.id,
                    title: cand.title,
                    similarity_score: abstract_score,
                    reason: format!("abstract similarity {:.2}", abstract_score),
                });
            }
        }
        Ok(out)
    }

    /// Append a contributor row, refusing the addition if it would push the
    /// running total of contribution shares above 100%.
    pub async fn add_contributor(
        &self,
        outcome_id: &str,
        input: AddContributorInput,
    ) -> AppResult<OutcomeContributor> {
        if !(0..=100).contains(&input.share_percentage) {
            return Err(AppError::Validation(
                "share_percentage must be between 0 and 100".into(),
            ));
        }
        let current_total: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(share_percentage), 0) FROM outcome_contributors WHERE outcome_id = ?",
        )
        .bind(outcome_id)
        .fetch_one(&self.db)
        .await?;
        if current_total + input.share_percentage > 100 {
            return Err(AppError::Validation(format!(
                "adding {}% would exceed the 100% cap (current total: {})",
                input.share_percentage, current_total
            )));
        }
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO outcome_contributors (id, outcome_id, user_id, share_percentage, role_in_work, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(outcome_id)
        .bind(&input.user_id)
        .bind(input.share_percentage)
        .bind(&input.role_in_work)
        .bind(&now)
        .execute(&self.db)
        .await?;
        let row = sqlx::query_as::<_, OutcomeContributor>(
            "SELECT * FROM outcome_contributors WHERE id = ?",
        )
        .bind(&id)
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    pub async fn list_contributors(&self, outcome_id: &str) -> AppResult<Vec<OutcomeContributor>> {
        let rows = sqlx::query_as::<_, OutcomeContributor>(
            "SELECT * FROM outcome_contributors WHERE outcome_id = ? ORDER BY created_at",
        )
        .bind(outcome_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    pub async fn remove_contributor(&self, contributor_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM outcome_contributors WHERE id = ?")
            .bind(contributor_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Submit an outcome for review. Refuses unless the contribution shares
    /// total **exactly** 100 — partial allocations are not allowed.
    pub async fn submit_outcome(&self, id: &str) -> AppResult<Outcome> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(share_percentage), 0) FROM outcome_contributors WHERE outcome_id = ?",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;
        if total != 100 {
            return Err(AppError::Validation(format!(
                "Contribution shares must total exactly 100% (got {})",
                total
            )));
        }
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE outcomes SET status = 'submitted', submitted_at = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;
        self.get_outcome(id).await
    }

    pub async fn approve_outcome(&self, id: &str, approver_id: &str) -> AppResult<Outcome> {
        let outcome = self.get_outcome(id).await?;
        if outcome.status != "submitted" {
            return Err(AppError::Conflict(
                "only submitted outcomes can be approved".into(),
            ));
        }
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE outcomes SET status='approved', approved_at = ?, approver_id = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(approver_id)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;
        self.record_approval_cycle(id, &outcome.submitted_at, Some(&now), approver_id)
            .await?;
        self.get_outcome(id).await
    }

    pub async fn reject_outcome(
        &self,
        id: &str,
        approver_id: &str,
        reason: &str,
    ) -> AppResult<Outcome> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE outcomes SET status='rejected', rejected_at = ?, approver_id = ?, rejection_reason = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&now)
        .bind(approver_id)
        .bind(reason)
        .bind(&now)
        .bind(id)
        .execute(&self.db)
        .await?;
        self.get_outcome(id).await
    }

    /// Insert one row in `approval_cycle_records` capturing the
    /// submitted_at → approved_at minute delta. Used by the analytics
    /// dashboard's approval-cycle histogram.
    pub async fn record_approval_cycle(
        &self,
        entity_id: &str,
        submitted_at: &Option<String>,
        approved_at: Option<&str>,
        approver_id: &str,
    ) -> AppResult<()> {
        let submitted = submitted_at.as_deref().unwrap_or("");
        let approved = approved_at.unwrap_or("");
        let cycle_minutes = match (
            DateTime::parse_from_rfc3339(submitted),
            DateTime::parse_from_rfc3339(approved),
        ) {
            (Ok(s), Ok(a)) => Some((a - s).num_minutes()),
            _ => None,
        };
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO approval_cycle_records (id, entity_type, entity_id, submitted_at, approved_at, approver_id, cycle_time_minutes)
             VALUES (?, 'outcome', ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(entity_id)
        .bind(submitted)
        .bind(approved)
        .bind(approver_id)
        .bind(cycle_minutes)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

/// Truncate a string to `max` chars (not bytes) for similarity comparison.
fn head(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}
