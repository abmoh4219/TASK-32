//! Backup HTTP handlers — admin-only.

use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;
use shared::AuditAction;

use crate::error::AppResult;
use crate::middleware::require_role::RequireAdmin;
use crate::models::backup::{BackupRecord, BackupSchedule, RetentionPolicy};
use crate::services::audit_service::AuditService;
use crate::services::backup_service::{BackupService, CleanupResult, SandboxValidationReport};
use crate::AppState;

fn db_path_from_url(url: &str) -> std::path::PathBuf {
    if let Some(rest) = url.strip_prefix("sqlite://") {
        std::path::PathBuf::from(rest)
    } else {
        std::path::PathBuf::from(url)
    }
}

fn build(state: &AppState) -> BackupService {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///app/data/scholarvault.db".to_string());
    BackupService::new(
        state.db.clone(),
        db_path_from_url(&db_url),
        (*state.evidence_dir).clone(),
        (*state.backup_dir).clone(),
        *state.encryption_key,
    )
}

pub async fn list_history(
    State(state): State<AppState>,
    RequireAdmin(_): RequireAdmin,
) -> AppResult<Json<Vec<BackupRecord>>> {
    Ok(Json(build(&state).list_backups().await?))
}

pub async fn run_backup(
    State(state): State<AppState>,
    RequireAdmin(user): RequireAdmin,
) -> AppResult<Json<BackupRecord>> {
    let svc = build(&state);
    let row = svc.run_backup().await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::RunBackup,
            "backup",
            Some(&row.id),
            Some(crate::services::audit_service::HASH_ENTITY_CREATED.to_string()),
            Some(AuditService::compute_hash(&row.sha256_hash)),
            None,
        )
        .await?;
    Ok(Json(row))
}

pub async fn restore_sandbox(
    State(state): State<AppState>,
    RequireAdmin(user): RequireAdmin,
    Path(id): Path<String>,
) -> AppResult<Json<SandboxValidationReport>> {
    let svc = build(&state);
    let report = svc.restore_to_sandbox(&id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::RestoreSandbox,
            "backup",
            Some(&id),
            Some(AuditService::compute_hash(&id)),
            Some(AuditService::compute_hash(&format!(
                "all_passed={}",
                report.all_passed
            ))),
            None,
        )
        .await?;
    Ok(Json(report))
}

pub async fn activate(
    State(state): State<AppState>,
    RequireAdmin(user): RequireAdmin,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = build(&state);
    svc.activate_restore(&id).await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::ActivateRestore,
            "backup",
            Some(&id),
            Some(AuditService::compute_hash(&id)),
            Some(AuditService::compute_hash(&format!("activated={}", id))),
            None,
        )
        .await?;
    Ok(Json(json!({"success": true})))
}

pub async fn cleanup(
    State(state): State<AppState>,
    RequireAdmin(user): RequireAdmin,
) -> AppResult<Json<CleanupResult>> {
    let svc = build(&state);
    let res = svc.apply_lifecycle_cleanup().await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::LifecycleCleanup,
            "backup",
            None,
            Some(AuditService::compute_hash("pre_cleanup")),
            Some(AuditService::compute_hash(&format!(
                "purged_d={} purged_m={}",
                res.purged_daily, res.purged_monthly
            ))),
            None,
        )
        .await?;
    Ok(Json(res))
}

pub async fn get_policy(
    State(state): State<AppState>,
    RequireAdmin(_): RequireAdmin,
) -> AppResult<Json<RetentionPolicy>> {
    Ok(Json(build(&state).get_active_policy().await?))
}

#[derive(serde::Deserialize)]
pub struct UpdatePolicyRequest {
    pub daily_retention: i64,
    pub monthly_retention: i64,
    pub preserve_financial: bool,
    pub preserve_ip: bool,
}

pub async fn get_schedule(
    State(state): State<AppState>,
    RequireAdmin(_): RequireAdmin,
) -> AppResult<Json<BackupSchedule>> {
    Ok(Json(build(&state).get_schedule().await?))
}

#[derive(serde::Deserialize)]
pub struct UpdateScheduleRequest {
    pub cron_expr: String,
}

pub async fn update_schedule(
    State(state): State<AppState>,
    RequireAdmin(user): RequireAdmin,
    Json(req): Json<UpdateScheduleRequest>,
) -> AppResult<Json<BackupSchedule>> {
    let svc = build(&state);
    // Capture before-state for audit before the mutation.
    let before_schedule = svc.get_schedule().await?;
    let before_hash = AuditService::compute_hash(&before_schedule.cron_expr);
    let updated = svc.update_schedule(&req.cron_expr, &user.id).await?;
    // Hot-reload the running scheduler so the new cron takes effect without a
    // full app restart. Failure to reload is logged but doesn't roll back the
    // persisted change — operators can retry or restart the process.
    if let Err(e) = crate::services::backup_scheduler::reload_scheduler(
        state.scheduler_handle.clone(),
        std::sync::Arc::new(svc),
        &updated.cron_expr,
    )
    .await
    {
        tracing::warn!(error = ?e, "backup scheduler reload failed — restart required");
    }
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "backup_schedule",
            Some(&updated.id),
            Some(before_hash),
            Some(AuditService::compute_hash(&updated.cron_expr)),
            None,
        )
        .await?;
    Ok(Json(updated))
}

pub async fn update_policy(
    State(state): State<AppState>,
    RequireAdmin(user): RequireAdmin,
    Json(req): Json<UpdatePolicyRequest>,
) -> AppResult<Json<RetentionPolicy>> {
    let svc = build(&state);
    let before_policy = svc.get_active_policy().await?;
    let before_hash = AuditService::compute_hash(&format!(
        "d={} m={} pf={} pi={}",
        before_policy.daily_retention,
        before_policy.monthly_retention,
        before_policy.preserve_financial,
        before_policy.preserve_ip
    ));
    let updated = svc
        .update_policy(
            req.daily_retention,
            req.monthly_retention,
            req.preserve_financial,
            req.preserve_ip,
            &user.id,
        )
        .await?;
    AuditService::new(state.db.clone())
        .log(
            &user.id,
            AuditAction::Update,
            "retention_policy",
            Some(&updated.id),
            Some(before_hash),
            Some(AuditService::compute_hash(&format!(
                "d={} m={} pf={} pi={}",
                updated.daily_retention,
                updated.monthly_retention,
                updated.preserve_financial,
                updated.preserve_ip
            ))),
            None,
        )
        .await?;
    Ok(Json(updated))
}
