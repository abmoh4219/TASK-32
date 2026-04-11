//! Outcome / IP registration row mappings.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Outcome {
    pub id: String,
    pub r#type: String,
    pub title: String,
    pub abstract_snippet: String,
    pub certificate_number: Option<String>,
    pub status: String,
    pub submitted_at: Option<String>,
    pub approved_at: Option<String>,
    pub rejected_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub approver_id: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutcomeContributor {
    pub id: String,
    pub outcome_id: String,
    pub user_id: String,
    pub share_percentage: i64,
    pub role_in_work: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceFile {
    pub id: String,
    pub outcome_id: String,
    pub filename: String,
    pub mime_type: String,
    pub stored_path: String,
    pub file_size: i64,
    pub sha256_fingerprint: String,
    pub uploaded_by: String,
    pub uploaded_at: String,
}
