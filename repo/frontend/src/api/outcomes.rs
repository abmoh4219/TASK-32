//! Outcomes API client wrappers.

use serde::{Deserialize, Serialize};

use crate::api::client::{get_json, post_json, ApiError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Outcome {
    pub id: String,
    #[serde(rename = "type")]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeContributor {
    pub id: String,
    pub outcome_id: String,
    pub user_id: String,
    pub share_percentage: i64,
    pub role_in_work: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize)]
pub struct CreateOutcomeInput {
    #[serde(rename = "type")]
    pub r#type: String,
    pub title: String,
    pub abstract_snippet: String,
    pub certificate_number: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddContributorInput {
    pub user_id: String,
    pub share_percentage: i64,
    pub role_in_work: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeWithEvidence {
    pub outcome: Outcome,
    pub contributors: Vec<OutcomeContributor>,
    pub evidence: Vec<EvidenceFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    pub a: Outcome,
    pub b: Outcome,
    pub title_similarity: f64,
    pub abstract_similarity: f64,
}

pub async fn list_outcomes() -> Result<Vec<Outcome>, ApiError> {
    get_json("/api/outcomes").await
}

pub async fn get_outcome(id: &str) -> Result<OutcomeWithEvidence, ApiError> {
    get_json(&format!("/api/outcomes/{}", id)).await
}

pub async fn create_outcome(input: CreateOutcomeInput) -> Result<CreateOutcomeResult, ApiError> {
    post_json("/api/outcomes", &input).await
}

pub async fn add_contributor(
    outcome_id: &str,
    input: AddContributorInput,
) -> Result<OutcomeContributor, ApiError> {
    post_json(&format!("/api/outcomes/{}/contributors", outcome_id), &input).await
}

pub async fn submit_outcome(id: &str) -> Result<Outcome, ApiError> {
    post_json(&format!("/api/outcomes/{}/submit", id), &serde_json::json!({})).await
}

pub async fn compare_outcomes(a: &str, b: &str) -> Result<CompareResult, ApiError> {
    get_json(&format!("/api/outcomes/{}/compare/{}", a, b)).await
}
