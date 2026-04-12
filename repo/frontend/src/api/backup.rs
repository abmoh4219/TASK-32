//! Backup / restore API client.

use serde::{Deserialize, Serialize};

use crate::api::client::{get_json, post_json, put_json, ApiError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: String,
    pub backup_type: String,
    pub bundle_path: String,
    pub sha256_hash: String,
    pub status: String,
    pub size_bytes: i64,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub restored_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxValidationReport {
    pub hash_ok: bool,
    pub integrity_ok: bool,
    pub read_test_ok: bool,
    pub all_passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub purged_daily: i64,
    pub purged_monthly: i64,
    pub preserved_financial: i64,
    pub preserved_ip: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: String,
    pub daily_retention: i64,
    pub monthly_retention: i64,
    pub preserve_financial: i64,
    pub preserve_ip: i64,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

pub async fn list_history() -> Result<Vec<BackupRecord>, ApiError> {
    get_json("/api/backup/history").await
}

pub async fn run_backup() -> Result<BackupRecord, ApiError> {
    post_json("/api/backup/run", &serde_json::json!({})).await
}

pub async fn restore_sandbox(id: &str) -> Result<SandboxValidationReport, ApiError> {
    post_json(&format!("/api/backup/{}/restore-sandbox", id), &serde_json::json!({})).await
}

pub async fn activate(id: &str) -> Result<serde_json::Value, ApiError> {
    post_json(&format!("/api/backup/{}/activate", id), &serde_json::json!({})).await
}

pub async fn lifecycle_cleanup() -> Result<CleanupResult, ApiError> {
    post_json("/api/backup/lifecycle-cleanup", &serde_json::json!({})).await
}

pub async fn get_policy() -> Result<RetentionPolicy, ApiError> {
    get_json("/api/backup/policy").await
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdatePolicyRequest {
    pub daily_retention: i64,
    pub monthly_retention: i64,
    pub preserve_financial: bool,
    pub preserve_ip: bool,
}

pub async fn update_policy(req: UpdatePolicyRequest) -> Result<RetentionPolicy, ApiError> {
    put_json("/api/backup/policy", &req).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub id: String,
    pub cron_expr: String,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

pub async fn get_schedule() -> Result<BackupSchedule, ApiError> {
    get_json("/api/backup/schedule").await
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateScheduleRequest {
    pub cron_expr: String,
}

pub async fn update_schedule(
    req: UpdateScheduleRequest,
) -> Result<BackupSchedule, ApiError> {
    put_json("/api/backup/schedule", &req).await
}
