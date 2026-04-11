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
