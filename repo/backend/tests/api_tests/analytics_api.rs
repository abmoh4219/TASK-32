//! Analytics HTTP integration tests — role gating, CSV export, scheduled report
//! single-use download token.

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
async fn test_fund_summary_finance_manager_allowed() {
    let (app, _state) = setup_test_app().await;
    let (session, _) = login_as(app.clone(), "finance", "Scholar2024!").await;
    let req = Request::builder()
        .uri("/api/analytics/funds")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_fund_summary_curator_forbidden() {
    let (app, _state) = setup_test_app().await;
    let (session, _) = login_as(app.clone(), "curator", "Scholar2024!").await;
    let req = Request::builder()
        .uri("/api/analytics/funds")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_export_csv_returns_text_csv() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "finance", "Scholar2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/analytics/export/csv")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({"report_type":"fund","period":null}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.contains("text/csv"), "got {}", ct);
    let bytes = to_bytes(resp.into_body(), 256 * 1024).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(text.starts_with("id,type,amount"));
}

#[tokio::test]
async fn test_scheduled_report_creates_complete_record() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "finance", "Scholar2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/analytics/reports/schedule")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({"report_type":"fund","format":"csv","period":null}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], "complete");
    assert!(body["download_token"].as_str().unwrap().len() > 8);
}

#[tokio::test]
async fn test_download_token_single_use_via_http() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "finance", "Scholar2024!").await;

    // Schedule a report
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/analytics/reports/schedule")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(
            json!({"report_type":"fund","format":"csv","period":null}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let id = body["id"].as_str().unwrap().to_string();
    let token = body["download_token"].as_str().unwrap().to_string();

    // First download — succeeds.
    let req = Request::builder()
        .uri(format!("/api/analytics/reports/{}/download/{}", id, token))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second download with same token — single-use cleared, expect 404.
    let req = Request::builder()
        .uri(format!("/api/analytics/reports/{}/download/{}", id, token))
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
