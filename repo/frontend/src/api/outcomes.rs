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

/// Upload an evidence file (PDF/JPG/PNG) to an existing outcome. Uses the
/// browser's `FormData` + `fetch` API to construct a real multipart/form-data
/// request that the backend's `Multipart` extractor expects.
///
/// Only compiled under WASM — native test targets cannot construct browser
/// FormData objects.
#[cfg(target_arch = "wasm32")]
pub async fn upload_evidence(
    outcome_id: &str,
    file: web_sys::File,
) -> Result<EvidenceFile, ApiError> {
    use wasm_bindgen::JsCast;
    let form = web_sys::FormData::new().map_err(|_| ApiError {
        status: 0,
        code: "JS_ERROR".into(),
        message: "FormData init failed".into(),
    })?;
    form.append_with_blob("file", &file).map_err(|_| ApiError {
        status: 0,
        code: "JS_ERROR".into(),
        message: "FormData append failed".into(),
    })?;
    let csrf = crate::api::client::read_csrf_cookie().unwrap_or_default();
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&form.into());
    let headers = web_sys::Headers::new().unwrap();
    headers.set("X-CSRF-Token", &csrf).ok();
    opts.set_headers(&headers.into());
    let url = format!("/api/outcomes/{}/evidence", outcome_id);
    let window = web_sys::window().unwrap();
    let resp_val = wasm_bindgen_futures::JsFuture::from(
        window.fetch_with_str_and_init(&url, &opts),
    )
    .await
    .map_err(|_| ApiError {
        status: 0,
        code: "NETWORK".into(),
        message: "fetch failed".into(),
    })?;
    let resp: web_sys::Response = resp_val.unchecked_into();
    let status = resp.status();
    let text = wasm_bindgen_futures::JsFuture::from(resp.text().unwrap())
        .await
        .map_err(|_| ApiError {
            status,
            code: "PARSE".into(),
            message: "body read failed".into(),
        })?
        .as_string()
        .unwrap_or_default();
    if !resp.ok() {
        let err: ApiError = serde_json::from_str(&text).unwrap_or(ApiError {
            status,
            code: "UNKNOWN".into(),
            message: "upload failed".into(),
        });
        return Err(err);
    }
    serde_json::from_str(&text).map_err(|e| ApiError {
        status,
        code: "PARSE".into(),
        message: format!("deserialize: {e}"),
    })
}
