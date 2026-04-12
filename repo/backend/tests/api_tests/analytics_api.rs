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
async fn test_exec_analytics_requires_finance_or_admin() {
    // Regression: members/churn/events/approval_cycles used to accept any
    // authenticated role. They now require RequireExecAnalytics (admin or
    // finance). Curator must be denied on all four and schedule_report.
    let (app, _state) = setup_test_app().await;
    let (c_sess, c_csrf) = login_as(app.clone(), "curator", "Scholar2024!").await;
    let c_cookie = format!("{}; csrf_token={}", c_sess, c_csrf);

    for path in [
        "/api/analytics/members",
        "/api/analytics/churn",
        "/api/analytics/events",
        "/api/analytics/approval-cycles",
    ] {
        let req = Request::builder()
            .uri(path)
            .header("cookie", c_cookie.clone())
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "curator should be denied on {}",
            path
        );
    }

    // schedule_report is also exec-only now.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/analytics/reports/schedule")
        .header("content-type", "application/json")
        .header("cookie", c_cookie.clone())
        .header("X-CSRF-Token", c_csrf.clone())
        .body(Body::from(
            json!({"report_type":"fund","format":"csv","period":null}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Finance manager still works on members.
    let (f_sess, _) = login_as(app.clone(), "finance", "Scholar2024!").await;
    let req = Request::builder()
        .uri("/api/analytics/members")
        .header("cookie", f_sess)
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

    let auth_cookie = format!("{}; csrf_token={}", session, csrf);

    // First download — succeeds (authenticated as report owner).
    let req = Request::builder()
        .uri(format!("/api/analytics/reports/{}/download/{}", id, token))
        .header("cookie", auth_cookie.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Second download with same token — single-use cleared, expect 404.
    let req = Request::builder()
        .uri(format!("/api/analytics/reports/{}/download/{}", id, token))
        .header("cookie", auth_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

/// Finding C regression: fund_summary endpoint supports custom query-string filters.
#[tokio::test]
async fn test_fund_summary_custom_filters() {
    let (app, _state) = setup_test_app().await;
    let (session, _) = login_as(app.clone(), "finance", "Scholar2024!").await;

    // Filter by category — only "grants" transactions.
    let req = Request::builder()
        .uri("/api/analytics/funds?category=grants")
        .header("cookie", session.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 128 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let txns = body["transactions"].as_array().unwrap();
    assert!(txns.iter().all(|t| t["category"] == "grants"),
        "filtered results should only contain 'grants' category");
}

/// Finding C regression: old unfiltered fund_summary call still works.
#[tokio::test]
async fn test_fund_summary_no_filter_backward_compat() {
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

/// Finding C regression: schedule_report accepts filter fields.
#[tokio::test]
async fn test_schedule_report_with_filters() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "finance", "Scholar2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/analytics/reports/schedule")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({
                "report_type":"fund",
                "format":"csv",
                "period":null,
                "date_from":"2026-01-01",
                "date_to":"2026-12-31",
                "category":"grants"
            }).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["status"], "complete");
    // Verify filters are stored in the filters JSON column.
    let filters: Value = serde_json::from_str(body["filters"].as_str().unwrap()).unwrap();
    assert_eq!(filters["category"], "grants");
}

#[tokio::test]
async fn test_download_report_requires_auth_and_ownership() {
    // Regression: the download endpoint used to be reachable by anyone in
    // possession of the token (no auth, no ownership check). We now require
    // an authenticated session AND creator match (admin bypass aside).
    let (app, _state) = setup_test_app().await;

    // Finance user schedules a report.
    let (f_sess, f_csrf) = login_as(app.clone(), "finance", "Scholar2024!").await;
    let f_cookie = format!("{}; csrf_token={}", f_sess, f_csrf);
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/analytics/reports/schedule")
        .header("content-type", "application/json")
        .header("cookie", f_cookie.clone())
        .header("X-CSRF-Token", f_csrf)
        .body(Body::from(
            json!({"report_type":"fund","format":"csv","period":null}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let id = body["id"].as_str().unwrap().to_string();
    let token = body["download_token"].as_str().unwrap().to_string();

    // Anonymous: no session cookie → 401.
    let req = Request::builder()
        .uri(format!("/api/analytics/reports/{}/download/{}", id, token))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Different user (admin is privileged, so pick a non-admin): curator holds
    // the (leaked) token but is not the creator → 403.
    let (c_sess, _) = login_as(app.clone(), "curator", "Scholar2024!").await;
    let req = Request::builder()
        .uri(format!("/api/analytics/reports/{}/download/{}", id, token))
        .header("cookie", c_sess)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Owner can still read it.
    let req = Request::builder()
        .uri(format!("/api/analytics/reports/{}/download/{}", id, token))
        .header("cookie", f_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
