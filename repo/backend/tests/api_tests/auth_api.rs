//! Auth HTTP integration tests — exercise login, CSRF, and the lockout policy
//! end-to-end against an in-memory SQLite + the real Axum router.

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::common::{setup_test_app, setup_test_db};

#[tokio::test]
async fn test_login_valid_credentials_returns_200() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"admin","password":"ScholarAdmin2024!"}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "valid login should return 200");

    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["username"], "admin");
    assert_eq!(body["role"], "administrator");
    assert!(body["csrf_token"].as_str().unwrap().len() >= 32);
}

#[tokio::test]
async fn test_login_wrong_password_returns_401() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"admin","password":"WRONG"}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_post_without_csrf_returns_403() {
    // /api/auth/refresh-csrf is not on the bootstrap whitelist, so a POST
    // with no X-CSRF-Token header must be rejected as CSRF_MISSING (403).
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/refresh-csrf")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_csrf_valid_token_passes_logout() {
    let (app, _state) = setup_test_app().await;

    let login_req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"curator","password":"Scholar2024!"}).to_string(),
        ))
        .unwrap();
    let login_resp = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(login_resp.status(), StatusCode::OK);

    let mut session_cookie = None;
    let mut csrf_cookie = None;
    for cookie in login_resp.headers().get_all("set-cookie").iter() {
        let v = cookie.to_str().unwrap_or("");
        if v.starts_with("sv_session=") {
            session_cookie = Some(v.split(';').next().unwrap().to_string());
        }
        if v.starts_with("csrf_token=") {
            csrf_cookie = Some(
                v.split(';')
                    .next()
                    .unwrap()
                    .strip_prefix("csrf_token=")
                    .unwrap()
                    .to_string(),
            );
        }
    }
    let session_cookie = session_cookie.expect("login should set sv_session");
    let csrf_token = csrf_cookie.expect("login should set csrf_token");

    let cookie_header = format!("{}; csrf_token={}", session_cookie, csrf_token);
    let logout_req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/logout")
        .header("cookie", cookie_header)
        .header("X-CSRF-Token", csrf_token)
        .body(Body::empty())
        .unwrap();
    let logout_resp = app.oneshot(logout_req).await.unwrap();
    assert_eq!(logout_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_lockout_blocks_after_5_failures() {
    let (app, _state) = setup_test_app().await;
    for _ in 0..5 {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"username":"admin","password":"wrong"}).to_string(),
            ))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"admin","password":"ScholarAdmin2024!"}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::LOCKED,
        "6th attempt within 15 minutes must be locked"
    );
}

#[tokio::test]
async fn test_seed_users_all_present() {
    let pool = setup_test_db().await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 5, "seed migration should have inserted 5 users");
}
