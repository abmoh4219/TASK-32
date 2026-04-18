//! Backup HTTP integration tests — admin-only role gating, run + restore flow.

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::common::setup_test_app;

async fn login_as(app: axum::Router, user: &str, pw: &str) -> (String, String) {
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username": user, "password": pw}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let mut session = String::new();
    let mut csrf = String::new();
    for h in resp.headers().get_all("set-cookie").iter() {
        let v = h.to_str().unwrap_or("");
        if let Some(rest) = v.strip_prefix("sv_session=") {
            session = format!("sv_session={}", rest.split(';').next().unwrap());
        }
        if let Some(rest) = v.strip_prefix("csrf_token=") {
            csrf = rest.split(';').next().unwrap().to_string();
        }
    }
    (session, csrf)
}

#[tokio::test]
async fn test_backup_run_creates_record() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/run")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], "complete");
    assert!(!body["sha256_hash"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_backup_admin_only() {
    let (app, _state) = setup_test_app().await;
    // Admin can access
    let (session, _) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let req = Request::builder()
        .uri("/api/backup/history")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_curator_cannot_access_backup() {
    let (app, _state) = setup_test_app().await;
    let (session, _) = login_as(app.clone(), "curator", "Scholar2024!").await;
    let req = Request::builder()
        .uri("/api/backup/history")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_restore_sandbox_returns_validation_report() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);

    // Run a backup
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/run")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let id = body["id"].as_str().unwrap().to_string();

    // Validate sandbox
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/backup/{}/restore-sandbox", id))
        .header("cookie", cookie)
        .header("X-CSRF-Token", csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let report: Value = serde_json::from_slice(&bytes).unwrap();
    // Hash check must succeed for a freshly-written bundle.
    assert_eq!(report["hash_ok"], true);
}

#[tokio::test]
async fn test_backup_run_produces_independent_db_and_files_records() {
    // Regression: a single run must now create two versioned records — one
    // tagged `database`, one tagged `files` — instead of a single combined
    // bundle.
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/run")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // List history and confirm both artifact kinds exist.
    let req = Request::builder()
        .uri("/api/backup/history")
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 128 * 1024).await.unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    let kinds: Vec<&str> = rows
        .iter()
        .filter_map(|r| r["artifact_kind"].as_str())
        .collect();
    assert!(
        kinds.contains(&"database"),
        "expected a `database` artifact in {:?}",
        kinds
    );
    assert!(
        kinds.contains(&"files"),
        "expected a `files` artifact in {:?}",
        kinds
    );
}

#[tokio::test]
async fn test_backup_schedule_admin_update_and_persist() {
    let (app, _state) = setup_test_app().await;
    let (a_sess, a_csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let a_cookie = format!("{}; csrf_token={}", a_sess, a_csrf);

    // GET default — seeded to "0 0 2 * * *".
    let req = Request::builder()
        .uri("/api/backup/schedule")
        .header("cookie", a_cookie.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 32 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["cron_expr"], "0 0 2 * * *");

    // Admin updates the cron expression.
    let req = Request::builder()
        .method(Method::PUT)
        .uri("/api/backup/schedule")
        .header("content-type", "application/json")
        .header("cookie", a_cookie.clone())
        .header("X-CSRF-Token", a_csrf.clone())
        .body(Body::from(json!({"cron_expr":"0 30 3 * * *"}).to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 32 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["cron_expr"], "0 30 3 * * *");

    // Read back — persisted.
    let req = Request::builder()
        .uri("/api/backup/schedule")
        .header("cookie", a_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = to_bytes(resp.into_body(), 32 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["cron_expr"], "0 30 3 * * *");

    // Non-admin (curator) rejected.
    let (c_sess, c_csrf) = login_as(app.clone(), "curator", "Scholar2024!").await;
    let req = Request::builder()
        .method(Method::PUT)
        .uri("/api/backup/schedule")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", c_sess, c_csrf))
        .header("X-CSRF-Token", c_csrf)
        .body(Body::from(json!({"cron_expr":"0 0 4 * * *"}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_restore_activation_applies_db_file_to_live_path() {
    // Regression: activation was a no-op that only stamped restored_at.
    // Drives BackupService against a real on-disk SQLite file with the full
    // migration suite so sandbox validation (integrity_check + SELECT users)
    // passes, then proves activation actually overwrites the live file.
    use backend::services::backup_service::BackupService;
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::fs;
    use std::str::FromStr;

    let (_, state) = setup_test_app().await;
    let tmp = std::env::temp_dir().join(format!("sv-restore-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&tmp).unwrap();
    let live_db = tmp.join("live.db");
    let live_evidence = tmp.join("evidence");
    fs::create_dir_all(&live_evidence).unwrap();

    // Build a real file-backed SQLite database with the full migration suite
    // seeded so the sandbox checks (integrity + users read) pass.
    let opts = SqliteConnectOptions::from_str(&format!(
        "sqlite://{}",
        live_db.to_string_lossy()
    ))
    .unwrap()
    .create_if_missing(true);
    let live_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .unwrap();
    let migrations_dir = format!(
        "{}/src/db/migrations",
        std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into())
    );
    backend::db::run_migrations(&live_pool, &migrations_dir)
        .await
        .unwrap();
    live_pool.close().await;

    let original_bytes = fs::read(&live_db).unwrap();
    assert!(original_bytes.len() > 1000, "seeded db should be non-trivial");

    let svc = BackupService::new(
        state.db.clone(),
        live_db.clone(),
        live_evidence.clone(),
        tmp.join("backups"),
        *state.encryption_key,
    );
    let db_record = svc.run_backup().await.unwrap();
    assert_eq!(db_record.artifact_kind.as_deref(), Some("database"));

    // Mutate the live file between backup and restore — we write a marker
    // tail so file length changes and content differs from the backup.
    let mut corrupted = original_bytes.clone();
    corrupted.extend_from_slice(b"AFTER-BACKUP-EDIT");
    fs::write(&live_db, &corrupted).unwrap();
    assert_ne!(fs::read(&live_db).unwrap(), original_bytes);

    // Activation must actually restore the live db file contents.
    svc.activate_restore(&db_record.id).await.unwrap();
    assert_eq!(
        fs::read(&live_db).unwrap(),
        original_bytes,
        "activation must overwrite the live db with the backed-up contents"
    );

    // Backup record now carries restored_at.
    let row: (Option<String>,) = sqlx::query_as(
        "SELECT restored_at FROM backup_records WHERE id = ?",
    )
    .bind(&db_record.id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert!(row.0.is_some(), "restored_at must be stamped after activation");
}

// ─── POST /api/backup/:id/activate ───────────────────────────────────────────

#[tokio::test]
async fn test_activate_backup_files_artifact_succeeds() {
    // The files artifact passes sandbox validation (hash + unpack only,
    // no live db integrity check), so activation succeeds in the test env.
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);

    // Run a backup.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/run")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Get history and find the files artifact.
    let req = Request::builder()
        .uri("/api/backup/history")
        .header("cookie", cookie.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = to_bytes(resp.into_body(), 128 * 1024).await.unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    let files_id = rows.iter()
        .find(|r| r["artifact_kind"] == "files")
        .and_then(|r| r["id"].as_str())
        .expect("files artifact must exist")
        .to_string();

    // Activate the files artifact.
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/backup/{}/activate", files_id))
        .header("cookie", cookie)
        .header("X-CSRF-Token", csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    // Files artifact activation succeeds (200) or may return 409 only for
    // database artifacts needing integrity_check. Accept 200 here.
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_activate_backup_anonymous_returns_401() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/some-id/activate")
        .header("cookie", "csrf_token=forged")
        .header("X-CSRF-Token", "forged")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_activate_backup_not_found_returns_404() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/nonexistent-backup-id/activate")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ─── POST /api/backup/lifecycle-cleanup ─────────────────────────────────────

#[tokio::test]
async fn test_lifecycle_cleanup_admin_succeeds() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/lifecycle-cleanup")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_lifecycle_cleanup_anonymous_returns_401() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/lifecycle-cleanup")
        .header("cookie", "csrf_token=forged")
        .header("X-CSRF-Token", "forged")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_lifecycle_cleanup_curator_forbidden() {
    let (app, _state) = setup_test_app().await;
    let (c_sess, c_csrf) = login_as(app.clone(), "curator", "Scholar2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/backup/lifecycle-cleanup")
        .header("cookie", format!("{}; csrf_token={}", c_sess, c_csrf))
        .header("X-CSRF-Token", c_csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_retention_policy_admin_can_update_others_cannot() {
    let (app, _state) = setup_test_app().await;
    let (a_sess, a_csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let a_cookie = format!("{}; csrf_token={}", a_sess, a_csrf);
    let payload = json!({
        "daily_retention": 45,
        "monthly_retention": 18,
        "preserve_financial": true,
        "preserve_ip": false
    })
    .to_string();

    // Admin updates the policy.
    let req = Request::builder()
        .method(Method::PUT)
        .uri("/api/backup/policy")
        .header("content-type", "application/json")
        .header("cookie", a_cookie.clone())
        .header("X-CSRF-Token", a_csrf.clone())
        .body(Body::from(payload.clone()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["daily_retention"], 45);
    assert_eq!(body["monthly_retention"], 18);
    assert_eq!(body["preserve_ip"], 0);

    // Re-read to confirm the change persisted.
    let req = Request::builder()
        .uri("/api/backup/policy")
        .header("cookie", a_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["daily_retention"], 45);

    // Non-admin (curator) must be denied.
    let (c_sess, c_csrf) = login_as(app.clone(), "curator", "Scholar2024!").await;
    let req = Request::builder()
        .method(Method::PUT)
        .uri("/api/backup/policy")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", c_sess, c_csrf))
        .header("X-CSRF-Token", c_csrf)
        .body(Body::from(payload))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
