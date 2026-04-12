//! Backup, restore, and lifecycle cleanup service.
//!
//! Bundles the SQLite database file plus the encrypted evidence directory into
//! a single tarball, AES-256-GCM encrypts the bytes, writes the result to
//! `<backup_dir>/<timestamp>-<type>.bin`, and records metadata in
//! `backup_records`. The "type" classification follows SPEC.md:
//!
//!   • the **last day of every calendar month** → `monthly`
//!   • everything else                          → `daily`
//!
//! Restore validation runs against a sandbox copy: the bundle is decrypted into
//! `/tmp`, the contained SQLite file is opened with a fresh pool, and a
//! `PRAGMA integrity_check` + `SELECT COUNT(*) FROM users` is executed plus the
//! SHA-256 of the bundle file is verified against the recorded hash. Only when
//! all three checks pass is the live database overwritten.

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::backup::{BackupRecord, BackupSchedule, RetentionPolicy};
use crate::security::encryption;

#[derive(Clone)]
pub struct BackupService {
    pub db: SqlitePool,
    pub db_path: PathBuf,
    pub evidence_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub encryption_key: [u8; 32],
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

impl BackupService {
    pub fn new(
        db: SqlitePool,
        db_path: PathBuf,
        evidence_dir: PathBuf,
        backup_dir: PathBuf,
        encryption_key: [u8; 32],
    ) -> Self {
        Self {
            db,
            db_path,
            evidence_dir,
            backup_dir,
            encryption_key,
        }
    }

    pub async fn list_backups(&self) -> AppResult<Vec<BackupRecord>> {
        let rows = sqlx::query_as::<_, BackupRecord>(
            "SELECT * FROM backup_records ORDER BY created_at DESC LIMIT 200",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }

    /// Create two independent versioned backup artifacts per run: one for the
    /// SQLite database and one for the uploaded-evidence directory. Each bundle
    /// is AES-256-GCM encrypted and gets its own `backup_records` row tagged
    /// with `artifact_kind`. Returns the database record for API compatibility
    /// (the files record can be listed via `list_backups`).
    pub async fn run_backup(&self) -> AppResult<BackupRecord> {
        fs::create_dir_all(&self.backup_dir).map_err(AppError::from)?;
        let now = Utc::now();
        let backup_type = if Self::is_last_day_of_month(now.date_naive()) {
            "monthly"
        } else {
            "daily"
        };

        // 1. Database artifact — contains scholarvault.db only.
        let db_id = Uuid::new_v4().to_string();
        let db_bundle_name = format!(
            "{}-{}-db-{}.bin",
            now.format("%Y%m%d"),
            backup_type,
            &db_id[..8]
        );
        let db_bundle_path = self.backup_dir.join(&db_bundle_name);
        let mut db_tar_buf: Vec<u8> = Vec::new();
        {
            let encoder = GzEncoder::new(&mut db_tar_buf, Compression::default());
            let mut tar = tar::Builder::new(encoder);
            if self.db_path.exists() {
                let mut file = File::open(&self.db_path).map_err(AppError::from)?;
                tar.append_file("scholarvault.db", &mut file)
                    .map_err(AppError::from)?;
            }
            tar.into_inner()
                .map_err(AppError::from)?
                .finish()
                .map_err(AppError::from)?;
        }
        let db_encrypted = encryption::encrypt_bytes(&db_tar_buf, &self.encryption_key)?;
        fs::write(&db_bundle_path, &db_encrypted).map_err(AppError::from)?;
        let db_hash = hex::encode(Sha256::digest(&db_encrypted));
        // Classification: the SQLite database carries the authoritative
        // financial (fund_transactions, orders, export_logs) AND IP
        // (outcomes, evidence_files metadata) records, so the database bundle
        // is marked as containing both categories for retention preservation.
        sqlx::query(
            "INSERT INTO backup_records (id, backup_type, bundle_path, sha256_hash, status, size_bytes, created_at, artifact_kind, contains_financial, contains_ip) VALUES (?, ?, ?, ?, 'complete', ?, ?, 'database', 1, 1)",
        )
        .bind(&db_id)
        .bind(backup_type)
        .bind(db_bundle_path.to_string_lossy().to_string())
        .bind(&db_hash)
        .bind(db_encrypted.len() as i64)
        .bind(now.to_rfc3339())
        .execute(&self.db)
        .await?;

        // 2. Files artifact — contains only the evidence directory. Recorded
        // as an independent versioned row so it can be verified and restored
        // separately from the database bundle.
        let files_id = Uuid::new_v4().to_string();
        let files_bundle_name = format!(
            "{}-{}-files-{}.bin",
            now.format("%Y%m%d"),
            backup_type,
            &files_id[..8]
        );
        let files_bundle_path = self.backup_dir.join(&files_bundle_name);
        let mut files_tar_buf: Vec<u8> = Vec::new();
        {
            let encoder = GzEncoder::new(&mut files_tar_buf, Compression::default());
            let mut tar = tar::Builder::new(encoder);
            if self.evidence_dir.exists() {
                tar.append_dir_all("evidence", &self.evidence_dir)
                    .map_err(AppError::from)?;
            }
            tar.into_inner()
                .map_err(AppError::from)?
                .finish()
                .map_err(AppError::from)?;
        }
        let files_encrypted =
            encryption::encrypt_bytes(&files_tar_buf, &self.encryption_key)?;
        fs::write(&files_bundle_path, &files_encrypted).map_err(AppError::from)?;
        let files_hash = hex::encode(Sha256::digest(&files_encrypted));
        // Files artifact holds evidence uploads which are IP records — but not
        // ledger/financial rows — so only the IP flag is set.
        sqlx::query(
            "INSERT INTO backup_records (id, backup_type, bundle_path, sha256_hash, status, size_bytes, created_at, artifact_kind, contains_financial, contains_ip) VALUES (?, ?, ?, ?, 'complete', ?, ?, 'files', 0, 1)",
        )
        .bind(&files_id)
        .bind(backup_type)
        .bind(files_bundle_path.to_string_lossy().to_string())
        .bind(&files_hash)
        .bind(files_encrypted.len() as i64)
        .bind(now.to_rfc3339())
        .execute(&self.db)
        .await?;

        let row = sqlx::query_as::<_, BackupRecord>(
            "SELECT * FROM backup_records WHERE id = ?",
        )
        .bind(&db_id)
        .fetch_one(&self.db)
        .await?;
        Ok(row)
    }

    /// Restore the bundle into a private sandbox directory and run the three
    /// validation checks. Does **not** touch the live database.
    pub async fn restore_to_sandbox(
        &self,
        backup_id: &str,
    ) -> AppResult<SandboxValidationReport> {
        let row = sqlx::query_as::<_, BackupRecord>(
            "SELECT * FROM backup_records WHERE id = ?",
        )
        .bind(backup_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::NotFound)?;

        let bundle_bytes = fs::read(&row.bundle_path).map_err(AppError::from)?;
        let actual_hash = hex::encode(Sha256::digest(&bundle_bytes));
        let hash_ok = actual_hash == row.sha256_hash;
        if !hash_ok {
            return Ok(SandboxValidationReport {
                hash_ok: false,
                integrity_ok: false,
                read_test_ok: false,
                all_passed: false,
                message: "SHA-256 of bundle does not match recorded hash".into(),
            });
        }

        // Decrypt + extract into a sandbox dir.
        let plaintext = encryption::decrypt_bytes(&bundle_bytes, &self.encryption_key)?;
        let sandbox_dir =
            std::env::temp_dir().join(format!("scholarvault-sandbox-{}", Uuid::new_v4()));
        fs::create_dir_all(&sandbox_dir).map_err(AppError::from)?;
        {
            let cursor = Cursor::new(&plaintext);
            let decoder = GzDecoder::new(cursor);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&sandbox_dir).map_err(AppError::from)?;
        }

        // A files-only artifact has no database to probe — hash + successful
        // unpack is the validation contract for that kind.
        if row.artifact_kind.as_deref() == Some("files") {
            return Ok(SandboxValidationReport {
                hash_ok: true,
                integrity_ok: true,
                read_test_ok: true,
                all_passed: true,
                message: "files artifact: hash + unpack validated".into(),
            });
        }

        let sandbox_db_path = sandbox_dir.join("scholarvault.db");
        if !sandbox_db_path.exists() {
            return Ok(SandboxValidationReport {
                hash_ok: true,
                integrity_ok: false,
                read_test_ok: false,
                all_passed: false,
                message: "scholarvault.db missing from bundle".into(),
            });
        }

        // Open a fresh pool against the sandbox file and run the two SQL checks.
        let db_url = format!("sqlite://{}", sandbox_db_path.display());
        let opts = SqliteConnectOptions::from_str(&db_url)
            .map_err(|e| AppError::Internal(e.to_string()))?
            .read_only(true);
        let sandbox_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .map_err(|e| AppError::Internal(format!("sandbox open: {e}")))?;

        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check;")
            .fetch_one(&sandbox_pool)
            .await
            .map_err(|e| AppError::Internal(format!("integrity_check: {e}")))?;
        let integrity_ok = integrity == "ok";

        let read_test: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&sandbox_pool)
            .await
            .map_err(|e| AppError::Internal(format!("read test: {e}")))?;
        let read_test_ok = read_test >= 0; // Any successful row count means the schema is intact.

        sandbox_pool.close().await;

        let all_passed = hash_ok && integrity_ok && read_test_ok;
        Ok(SandboxValidationReport {
            hash_ok,
            integrity_ok,
            read_test_ok,
            all_passed,
            message: if all_passed {
                "all sandbox checks passed".into()
            } else {
                "one or more sandbox checks failed".into()
            },
        })
    }

    /// Re-validate the sandbox and **atomically apply** the artifact to live
    /// storage. Database artifacts overwrite the live SQLite file; files
    /// artifacts replace the evidence directory contents. Before mutating
    /// anything we snapshot the current live state to a sibling
    /// `.pre-restore-<ts>` location so any failure rolls back cleanly. Refuses
    /// when sandbox validation didn't fully pass.
    pub async fn activate_restore(&self, backup_id: &str) -> AppResult<()> {
        let report = self.restore_to_sandbox(backup_id).await?;
        if !report.all_passed {
            return Err(AppError::Conflict(
                "sandbox validation must pass before activation".into(),
            ));
        }
        let row = sqlx::query_as::<_, BackupRecord>(
            "SELECT * FROM backup_records WHERE id = ?",
        )
        .bind(backup_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::NotFound)?;

        // Re-extract the bundle into a sandbox directory. We already know the
        // hash/integrity checks passed above, so this is just for the apply.
        let bundle_bytes = fs::read(&row.bundle_path).map_err(AppError::from)?;
        let plaintext = encryption::decrypt_bytes(&bundle_bytes, &self.encryption_key)?;
        let sandbox_dir =
            std::env::temp_dir().join(format!("scholarvault-apply-{}", Uuid::new_v4()));
        fs::create_dir_all(&sandbox_dir).map_err(AppError::from)?;
        {
            let cursor = Cursor::new(&plaintext);
            let decoder = GzDecoder::new(cursor);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&sandbox_dir).map_err(AppError::from)?;
        }

        let ts = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let kind = row.artifact_kind.as_deref().unwrap_or("database");
        match kind {
            "database" => {
                let sandbox_db = sandbox_dir.join("scholarvault.db");
                if !sandbox_db.exists() {
                    return Err(AppError::Internal(
                        "sandbox missing scholarvault.db".into(),
                    ));
                }
                // Rollback guard: copy the live db aside before overwriting.
                let rollback_path = self
                    .db_path
                    .with_extension(format!("pre-restore-{ts}.db"));
                if self.db_path.exists() {
                    fs::copy(&self.db_path, &rollback_path).map_err(AppError::from)?;
                }
                if let Err(e) = fs::copy(&sandbox_db, &self.db_path) {
                    // Best-effort rollback.
                    if rollback_path.exists() {
                        let _ = fs::copy(&rollback_path, &self.db_path);
                    }
                    return Err(AppError::Internal(format!(
                        "failed to activate database artifact: {e}"
                    )));
                }
            }
            "files" => {
                let sandbox_evidence = sandbox_dir.join("evidence");
                if !sandbox_evidence.exists() {
                    return Err(AppError::Internal(
                        "sandbox missing evidence directory".into(),
                    ));
                }
                // Rollback guard: rename current evidence dir aside.
                let rollback_dir = self
                    .evidence_dir
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("/tmp"))
                    .join(format!(
                        "{}-pre-restore-{ts}",
                        self.evidence_dir
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("evidence")
                    ));
                if self.evidence_dir.exists() {
                    let _ = fs::rename(&self.evidence_dir, &rollback_dir);
                }
                if let Err(e) = copy_dir_recursive(&sandbox_evidence, &self.evidence_dir)
                {
                    // Rollback: put the old directory back.
                    let _ = fs::remove_dir_all(&self.evidence_dir);
                    if rollback_dir.exists() {
                        let _ = fs::rename(&rollback_dir, &self.evidence_dir);
                    }
                    return Err(AppError::Internal(format!(
                        "failed to activate files artifact: {e}"
                    )));
                }
            }
            other => {
                return Err(AppError::Validation(format!(
                    "unknown artifact_kind: {other}"
                )));
            }
        }

        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE backup_records SET restored_at = ? WHERE id = ?")
            .bind(&now)
            .bind(backup_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // ─── Admin-configurable schedule ─────────────────────────────────────

    pub async fn get_schedule(&self) -> AppResult<BackupSchedule> {
        let row = sqlx::query_as::<_, BackupSchedule>(
            "SELECT * FROM backup_schedules WHERE id = 'default'",
        )
        .fetch_optional(&self.db)
        .await?;
        row.ok_or_else(|| AppError::Internal("backup schedule missing".into()))
    }

    pub async fn update_schedule(
        &self,
        cron_expr: &str,
        actor_id: &str,
    ) -> AppResult<BackupSchedule> {
        let trimmed = cron_expr.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("cron_expr is required".into()));
        }
        // Minimal sanity check: tokio_cron_scheduler uses 6- or 7-field cron
        // (sec min hour dom mon dow [year]). Reject anything shorter so the
        // scheduler doesn't crash on reload.
        let field_count = trimmed.split_whitespace().count();
        if !(5..=7).contains(&field_count) {
            return Err(AppError::Validation(format!(
                "cron_expr must have 5-7 fields, got {field_count}"
            )));
        }
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE backup_schedules SET cron_expr = ?, updated_at = ?, updated_by = ? WHERE id = 'default'",
        )
        .bind(trimmed)
        .bind(&now)
        .bind(actor_id)
        .execute(&self.db)
        .await?;
        self.get_schedule().await
    }

    /// Apply the active retention policy: delete daily backups older than
    /// `daily_retention` days and monthly backups older than
    /// `monthly_retention` months. The `preserve_financial` / `preserve_ip`
    /// flags are honoured by counting (and skipping) any record marked with
    /// those statuses.
    pub async fn apply_lifecycle_cleanup(&self) -> AppResult<CleanupResult> {
        let policy = self.get_active_policy().await?;
        let daily_cutoff = Utc::now() - Duration::days(policy.daily_retention);
        let monthly_cutoff = Utc::now() - Duration::days(policy.monthly_retention * 30);

        let mut purged_daily = 0i64;
        let mut purged_monthly = 0i64;
        let mut preserved_financial = 0i64;
        let mut preserved_ip = 0i64;

        let candidates = sqlx::query_as::<_, BackupRecord>(
            "SELECT * FROM backup_records WHERE status = 'complete'",
        )
        .fetch_all(&self.db)
        .await?;

        for r in candidates {
            let created = match DateTime::parse_from_rfc3339(&r.created_at) {
                Ok(d) => d.with_timezone(&Utc),
                Err(_) => continue,
            };
            let too_old = match r.backup_type.as_str() {
                "daily" => created < daily_cutoff,
                "monthly" => created < monthly_cutoff,
                _ => false,
            };
            if !too_old {
                continue;
            }
            // Preserve from structured classification metadata (no filename
            // guessing): if the record is flagged as containing financial or
            // IP data and the policy asks to preserve that category, we skip
            // the purge and increment the preserved counter.
            if policy.preserve_financial == 1 && r.contains_financial == 1 {
                preserved_financial += 1;
                continue;
            }
            if policy.preserve_ip == 1 && r.contains_ip == 1 {
                preserved_ip += 1;
                continue;
            }
            let _ = fs::remove_file(&r.bundle_path);
            sqlx::query("UPDATE backup_records SET status = 'purged' WHERE id = ?")
                .bind(&r.id)
                .execute(&self.db)
                .await?;
            match r.backup_type.as_str() {
                "daily" => purged_daily += 1,
                "monthly" => purged_monthly += 1,
                _ => {}
            }
        }
        Ok(CleanupResult {
            purged_daily,
            purged_monthly,
            preserved_financial,
            preserved_ip,
        })
    }

    pub async fn get_active_policy(&self) -> AppResult<RetentionPolicy> {
        let row = sqlx::query_as::<_, RetentionPolicy>(
            "SELECT * FROM retention_policies WHERE id = 'default'",
        )
        .fetch_optional(&self.db)
        .await?;
        row.ok_or_else(|| AppError::Internal("retention policy missing".into()))
    }

    /// Admin mutation: update the single active retention policy row.
    /// Validates basic ranges and stamps the actor for audit.
    pub async fn update_policy(
        &self,
        daily_retention: i64,
        monthly_retention: i64,
        preserve_financial: bool,
        preserve_ip: bool,
        actor_id: &str,
    ) -> AppResult<RetentionPolicy> {
        if !(1..=3650).contains(&daily_retention) {
            return Err(AppError::Validation(
                "daily_retention must be between 1 and 3650 days".into(),
            ));
        }
        if !(1..=120).contains(&monthly_retention) {
            return Err(AppError::Validation(
                "monthly_retention must be between 1 and 120 months".into(),
            ));
        }
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "UPDATE retention_policies SET daily_retention = ?, monthly_retention = ?, \
             preserve_financial = ?, preserve_ip = ?, updated_at = ?, updated_by = ? \
             WHERE id = 'default'",
        )
        .bind(daily_retention)
        .bind(monthly_retention)
        .bind(if preserve_financial { 1i64 } else { 0 })
        .bind(if preserve_ip { 1i64 } else { 0 })
        .bind(&now)
        .bind(actor_id)
        .execute(&self.db)
        .await?;
        self.get_active_policy().await
    }

    /// Returns true when `date` is the last day of its calendar month.
    pub fn is_last_day_of_month(date: NaiveDate) -> bool {
        let next = date.succ_opt();
        match next {
            Some(d) => d.month() != date.month(),
            None => true,
        }
    }
}

/// Recursive directory copy used by the files-artifact restore path. Creates
/// the destination if missing, then walks the source tree copying every
/// regular file. Not fancy: this is a narrow helper for activation, not a
/// general-purpose utility.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}
