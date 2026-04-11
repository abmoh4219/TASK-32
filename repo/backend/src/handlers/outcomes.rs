//! Outcome / IP HTTP handlers — list, register, contributors, evidence upload,
//! submit, side-by-side compare. Every mutation writes an audit log row.

use axum::{
    extract::{Multipart, Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared::AuditAction;

use crate::error::{AppError, AppResult};
use crate::middleware::require_role::{AuthenticatedUser, RequireReviewer};
use crate::models::outcome::{EvidenceFile, Outcome, OutcomeContributor};
use crate::services::audit_service::AuditService;
use crate::services::file_service::FileService;
use crate::services::outcome_service::{
    AddContributorInput, CreateOutcomeInput, CreateOutcomeResult, DuplicateCandidate,
    OutcomeService,
};
use crate::AppState;

pub async fn list_outcomes(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> AppResult<Json<Vec<Outcome>>> {
    let svc = OutcomeService::new(state.db.clone());
    Ok(Json(svc.list_outcomes(200).await?))
}

pub async fn get_outcome(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(id): Path<String>,
) -> AppResult<Json<OutcomeWithEvidence>> {
    let svc = OutcomeService::new(state.db.clone());
    let outcome = svc.get_outcome(&id).await?;
    let contributors = svc.list_contributors(&id).await?;
    let file_svc = FileService::new(
        state.db.clone(),
        *state.encryption_key,
        (*state.evidence_dir).clone(),
    );
    let evidence = file_svc.list_for_outcome(&id).await?;
    Ok(Json(OutcomeWithEvidence {
        outcome,
        contributors,
        evidence,
    }))
}

#[derive(Debug, Serialize)]
pub struct OutcomeWithEvidence {
    pub outcome: Outcome,
    pub contributors: Vec<OutcomeContributor>,
    pub evidence: Vec<EvidenceFile>,
}

pub async fn create_outcome(
    State(state): State<AppState>,
    RequireReviewer(user): RequireReviewer,
    Json(input): Json<CreateOutcomeInput>,
) -> AppResult<Json<CreateOutcomeResult>> {
    let svc = OutcomeService::new(state.db.clone());
    let result = svc.create_outcome(input, &user.id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Create,
            "outcome",
            Some(&result.outcome.id),
            None,
            Some(AuditService::compute_hash(
                &serde_json::to_string(&result.outcome)?,
            )),
            None,
        )
        .await?;
    Ok(Json(result))
}

pub async fn add_contributor(
    State(state): State<AppState>,
    RequireReviewer(user): RequireReviewer,
    Path(outcome_id): Path<String>,
    Json(input): Json<AddContributorInput>,
) -> AppResult<Json<OutcomeContributor>> {
    let svc = OutcomeService::new(state.db.clone());
    let row = svc.add_contributor(&outcome_id, input).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "outcome_contributor",
            Some(&outcome_id),
            None,
            Some(AuditService::compute_hash(&serde_json::to_string(&row)?)),
            None,
        )
        .await?;
    Ok(Json(row))
}

pub async fn remove_contributor(
    State(state): State<AppState>,
    RequireReviewer(user): RequireReviewer,
    Path((outcome_id, contributor_id)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = OutcomeService::new(state.db.clone());
    svc.remove_contributor(&contributor_id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Delete,
            "outcome_contributor",
            Some(&outcome_id),
            Some(AuditService::compute_hash(&contributor_id)),
            None,
            None,
        )
        .await?;
    Ok(Json(json!({"success": true})))
}

pub async fn submit_outcome(
    State(state): State<AppState>,
    RequireReviewer(user): RequireReviewer,
    Path(id): Path<String>,
) -> AppResult<Json<Outcome>> {
    let svc = OutcomeService::new(state.db.clone());
    let updated = svc.submit_outcome(&id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Submit,
            "outcome",
            Some(&id),
            None,
            Some(AuditService::compute_hash(&serde_json::to_string(&updated)?)),
            None,
        )
        .await?;
    Ok(Json(updated))
}

#[derive(Deserialize)]
pub struct ApprovalRequest {
    pub reason: Option<String>,
}

pub async fn approve_outcome(
    State(state): State<AppState>,
    RequireReviewer(user): RequireReviewer,
    Path(id): Path<String>,
) -> AppResult<Json<Outcome>> {
    let svc = OutcomeService::new(state.db.clone());
    let updated = svc.approve_outcome(&id, &user.id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Approve,
            "outcome",
            Some(&id),
            None,
            Some(AuditService::compute_hash(&serde_json::to_string(&updated)?)),
            None,
        )
        .await?;
    Ok(Json(updated))
}

pub async fn reject_outcome(
    State(state): State<AppState>,
    RequireReviewer(user): RequireReviewer,
    Path(id): Path<String>,
    Json(req): Json<ApprovalRequest>,
) -> AppResult<Json<Outcome>> {
    let svc = OutcomeService::new(state.db.clone());
    let reason = req.reason.unwrap_or_else(|| "no reason supplied".into());
    let updated = svc.reject_outcome(&id, &user.id, &reason).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Reject,
            "outcome",
            Some(&id),
            None,
            Some(AuditService::compute_hash(&serde_json::to_string(&updated)?)),
            None,
        )
        .await?;
    Ok(Json(updated))
}

pub async fn upload_evidence(
    State(state): State<AppState>,
    RequireReviewer(user): RequireReviewer,
    Path(outcome_id): Path<String>,
    mut multipart: Multipart,
) -> AppResult<Json<EvidenceFile>> {
    let mut bytes_field: Option<Vec<u8>> = None;
    let mut filename = String::from("upload.bin");
    let mut declared_mime = String::from("application/octet-stream");

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            if let Some(fname) = field.file_name() {
                filename = fname.to_string();
            }
            if let Some(ct) = field.content_type() {
                declared_mime = ct.to_string();
            }
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Validation(format!("multipart bytes: {e}")))?;
            bytes_field = Some(data.to_vec());
        }
    }
    let bytes = bytes_field.ok_or_else(|| AppError::Validation("missing 'file' field".into()))?;

    let file_svc = FileService::new(
        state.db.clone(),
        *state.encryption_key,
        (*state.evidence_dir).clone(),
    );
    let row = file_svc
        .upload_evidence(&outcome_id, &bytes, &filename, &declared_mime, &user.id)
        .await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::UploadEvidence,
            "evidence_file",
            Some(&row.id),
            None,
            Some(AuditService::compute_hash(&row.sha256_fingerprint)),
            None,
        )
        .await?;
    Ok(Json(row))
}

pub async fn compare_outcomes(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path((id_a, id_b)): Path<(String, String)>,
) -> AppResult<Json<CompareResult>> {
    let svc = OutcomeService::new(state.db.clone());
    let a = svc.get_outcome(&id_a).await?;
    let b = svc.get_outcome(&id_b).await?;
    let title_score = strsim::jaro_winkler(&a.title, &b.title);
    let abstract_score = strsim::jaro_winkler(&a.abstract_snippet, &b.abstract_snippet);
    Ok(Json(CompareResult {
        a,
        b,
        title_similarity: title_score,
        abstract_similarity: abstract_score,
    }))
}

#[derive(Debug, Serialize)]
pub struct CompareResult {
    pub a: Outcome,
    pub b: Outcome,
    pub title_similarity: f64,
    pub abstract_similarity: f64,
}

#[allow(dead_code)]
fn _force_use(_: DuplicateCandidate) {}
