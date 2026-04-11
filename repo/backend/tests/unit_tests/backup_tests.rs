//! Backend unit tests for the backup service.

use std::str::FromStr;

use backend::services::backup_service::BackupService;
use chrono::{Duration, NaiveDate, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

async fn fresh_db() -> SqlitePool {
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .unwrap();
    let dir = format!("{}/src/db/migrations", env!("CARGO_MANIFEST_DIR"));
    backend::db::run_migrations(&pool, &dir).await.unwrap();
    pool
}

fn build(pool: SqlitePool, db_path: std::path::PathBuf) -> BackupService {
    let tmp = std::env::temp_dir().join(format!("sv-bk-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(tmp.join("evidence")).unwrap();
    std::fs::create_dir_all(tmp.join("backups")).unwrap();
    BackupService::new(
        pool,
        db_path,
        tmp.join("evidence"),
        tmp.join("backups"),
        [7u8; 32],
    )
}

#[test]
fn test_daily_backup_type_mid_month() {
    // 2026-04-15 is not the last day of April → daily.
    let date = NaiveDate::from_ymd_opt(2026, 4, 15).unwrap();
    assert!(!BackupService::is_last_day_of_month(date));
}

#[test]
fn test_monthly_backup_type_last_day_of_month() {
    let date = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();
    assert!(BackupService::is_last_day_of_month(date));
    let feb_leap = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
    assert!(BackupService::is_last_day_of_month(feb_leap));
}

#[tokio::test]
async fn test_run_backup_creates_record() {
    let pool = fresh_db().await;
    // Use the in-memory db marker as the path — run_backup will skip
    // appending the file because the path doesn't exist on disk, which is
    // fine for the test (the resulting bundle still contains the evidence
    // dir + a hash + a record row).
    let svc = build(pool.clone(), std::path::PathBuf::from(":memory:"));
    let row = svc.run_backup().await.unwrap();
    assert!(row.bundle_path.ends_with(".bin"));
    assert_eq!(row.status, "complete");
    assert!(!row.sha256_hash.is_empty());
    assert!(std::path::Path::new(&row.bundle_path).exists());
}

#[tokio::test]
async fn test_lifecycle_cleanup_removes_old_daily() {
    let pool = fresh_db().await;
    // Insert a daily record dated 60 days ago + a fake bundle file.
    let svc = build(pool.clone(), std::path::PathBuf::from(":memory:"));
    let old_date = (Utc::now() - Duration::days(60)).to_rfc3339();
    let bundle = svc.backup_dir.join("old-daily.bin");
    std::fs::write(&bundle, b"test").unwrap();
    sqlx::query(
        "INSERT INTO backup_records (id, backup_type, bundle_path, sha256_hash, status, size_bytes, created_at) VALUES (?, 'daily', ?, ?, 'complete', 4, ?)",
    )
    .bind("old-1")
    .bind(bundle.to_string_lossy().to_string())
    .bind("hash")
    .bind(&old_date)
    .execute(&pool)
    .await
    .unwrap();
    let res = svc.apply_lifecycle_cleanup().await.unwrap();
    assert_eq!(res.purged_daily, 1);
    assert!(!bundle.exists());
}

#[tokio::test]
async fn test_lifecycle_cleanup_removes_old_monthly() {
    let pool = fresh_db().await;
    let svc = build(pool.clone(), std::path::PathBuf::from(":memory:"));
    let old_date = (Utc::now() - Duration::days(400)).to_rfc3339();
    let bundle = svc.backup_dir.join("old-monthly.bin");
    std::fs::write(&bundle, b"test").unwrap();
    sqlx::query(
        "INSERT INTO backup_records (id, backup_type, bundle_path, sha256_hash, status, size_bytes, created_at) VALUES (?, 'monthly', ?, ?, 'complete', 4, ?)",
    )
    .bind("old-monthly-1")
    .bind(bundle.to_string_lossy().to_string())
    .bind("hash")
    .bind(&old_date)
    .execute(&pool)
    .await
    .unwrap();
    let res = svc.apply_lifecycle_cleanup().await.unwrap();
    assert_eq!(res.purged_monthly, 1);
}

#[tokio::test]
async fn test_lifecycle_cleanup_preserves_recent() {
    let pool = fresh_db().await;
    let svc = build(pool.clone(), std::path::PathBuf::from(":memory:"));
    let recent = Utc::now().to_rfc3339();
    let bundle = svc.backup_dir.join("recent.bin");
    std::fs::write(&bundle, b"test").unwrap();
    sqlx::query(
        "INSERT INTO backup_records (id, backup_type, bundle_path, sha256_hash, status, size_bytes, created_at) VALUES (?, 'daily', ?, ?, 'complete', 4, ?)",
    )
    .bind("recent-1")
    .bind(bundle.to_string_lossy().to_string())
    .bind("hash")
    .bind(&recent)
    .execute(&pool)
    .await
    .unwrap();
    let res = svc.apply_lifecycle_cleanup().await.unwrap();
    assert_eq!(res.purged_daily, 0);
    assert!(bundle.exists(), "recent backup must NOT be purged");
}

#[tokio::test]
async fn test_lifecycle_cleanup_preserves_financial_marker() {
    let pool = fresh_db().await;
    let svc = build(pool.clone(), std::path::PathBuf::from(":memory:"));
    // Bundle path contains "financial" → preserved by policy.preserve_financial=1.
    let old_date = (Utc::now() - Duration::days(60)).to_rfc3339();
    let bundle = svc.backup_dir.join("financial-archive.bin");
    std::fs::write(&bundle, b"test").unwrap();
    sqlx::query(
        "INSERT INTO backup_records (id, backup_type, bundle_path, sha256_hash, status, size_bytes, created_at) VALUES (?, 'daily', ?, ?, 'complete', 4, ?)",
    )
    .bind("fin-1")
    .bind(bundle.to_string_lossy().to_string())
    .bind("hash")
    .bind(&old_date)
    .execute(&pool)
    .await
    .unwrap();
    let res = svc.apply_lifecycle_cleanup().await.unwrap();
    assert_eq!(res.preserved_financial, 1);
    assert_eq!(res.purged_daily, 0);
    assert!(bundle.exists());
}

#[tokio::test]
async fn test_restore_sandbox_sha256_verification() {
    let pool = fresh_db().await;
    let svc = build(pool.clone(), std::path::PathBuf::from(":memory:"));
    // Run a real backup so the sandbox restore has a valid bundle to read.
    let row = svc.run_backup().await.unwrap();
    let report = svc.restore_to_sandbox(&row.id).await.unwrap();
    assert!(report.hash_ok, "freshly created bundle must validate");
    // The sandbox unpack will succeed (empty evidence dir + missing live db),
    // but the missing scholarvault.db means integrity_check + read_test fail,
    // which is the expected validation behaviour.
    assert!(!report.all_passed);
}

#[tokio::test]
async fn test_restore_sandbox_tampered_bundle_fails() {
    let pool = fresh_db().await;
    let svc = build(pool.clone(), std::path::PathBuf::from(":memory:"));
    let row = svc.run_backup().await.unwrap();
    // Tamper with the bundle on disk.
    std::fs::write(&row.bundle_path, b"tampered bytes here").unwrap();
    let report = svc.restore_to_sandbox(&row.id).await.unwrap();
    assert!(!report.hash_ok);
    assert!(!report.all_passed);
}
