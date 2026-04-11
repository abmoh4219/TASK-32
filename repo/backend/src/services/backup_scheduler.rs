//! Cron-driven backup scheduler.
//!
//! Reads the `BACKUP_SCHEDULE` env var (default `"0 2 * * *"` — 02:00 daily)
//! and registers a `tokio_cron_scheduler` job that calls
//! `BackupService::run_backup` on the configured cadence. The scheduler is
//! started from `main.rs`; integration tests skip it.

use std::sync::Arc;

use tokio_cron_scheduler::{Job, JobScheduler};

use crate::error::{AppError, AppResult};
use crate::services::backup_service::BackupService;

/// Spawn the cron scheduler. Returns the running `JobScheduler` so the caller
/// can keep it alive for the lifetime of the process.
pub async fn start_backup_scheduler(
    service: Arc<BackupService>,
    cron_expr: &str,
) -> AppResult<JobScheduler> {
    let scheduler = JobScheduler::new()
        .await
        .map_err(|e| AppError::Internal(format!("scheduler init: {e}")))?;
    let svc = service.clone();
    let expr = cron_expr.to_string();
    let job = Job::new_async(expr.as_str(), move |_uuid, _l| {
        let svc = svc.clone();
        Box::pin(async move {
            match svc.run_backup().await {
                Ok(rec) => tracing::info!(id = %rec.id, ty = %rec.backup_type, "backup complete"),
                Err(e) => tracing::error!(error = ?e, "scheduled backup failed"),
            }
        })
    })
    .map_err(|e| AppError::Internal(format!("scheduler job: {e}")))?;
    scheduler
        .add(job)
        .await
        .map_err(|e| AppError::Internal(format!("scheduler add: {e}")))?;
    scheduler
        .start()
        .await
        .map_err(|e| AppError::Internal(format!("scheduler start: {e}")))?;
    tracing::info!(cron = %cron_expr, "backup scheduler started");
    Ok(scheduler)
}
