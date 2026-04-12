//! Shared helpers for API integration tests: build a fresh in-memory SQLite
//! database, run the migration suite, and return a router + state ready for
//! `tower::ServiceExt::oneshot` calls.

use std::str::FromStr;
use std::sync::Arc;

use backend::middleware::rate_limit::RateLimitState;
use backend::{db, derive_key, router::build_router, AppState};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

#[allow(dead_code)]
pub async fn setup_test_app() -> (axum::Router, AppState) {
    let pool = setup_test_db().await;
    let tmp_root = std::env::temp_dir().join(format!("scholarvault-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(tmp_root.join("evidence")).ok();
    std::fs::create_dir_all(tmp_root.join("backups")).ok();
    std::fs::create_dir_all(tmp_root.join("reports")).ok();
    let state = AppState {
        db: pool.clone(),
        encryption_key: Arc::new(derive_key("test-encryption-key-exactly-32bytes")),
        signing_key: Arc::new("test-signing-key".to_string()),
        rate_limit: RateLimitState::new(60),
        evidence_dir: Arc::new(tmp_root.join("evidence")),
        backup_dir: Arc::new(tmp_root.join("backups")),
        reports_dir: Arc::new(tmp_root.join("reports")),
        invalid_search_tracker: backend::services::abuse::InvalidSearchTracker::new(),
        scheduler_handle: Arc::new(tokio::sync::Mutex::new(None)),
    };
    let app = build_router(state.clone());
    (app, state)
}

#[allow(dead_code)]
pub async fn setup_test_db() -> SqlitePool {
    // Single in-memory connection so migrations + queries share the same db.
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .expect("open in-memory sqlite");

    let migrations_dir = format!("{}/src/db/migrations", env!("CARGO_MANIFEST_DIR"));
    db::run_migrations(&pool, &migrations_dir)
        .await
        .expect("migrations");

    pool
}

#[allow(dead_code)]
pub fn unused_marker() {
    // Force the helpers to be linked even if a particular test file does not
    // import them yet — keeps the warning surface clean during early phases.
    let _ = setup_test_app;
    let _ = setup_test_db;
}
