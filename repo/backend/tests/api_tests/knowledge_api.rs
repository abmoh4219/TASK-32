//! Knowledge HTTP integration tests — exercises real Axum routes against an
//! in-memory SQLite to confirm role gates, CSRF, and merge-cycle conflict.

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::common::setup_test_app;

async fn login_as(app: axum::Router, username: &str, password: &str) -> (String, String) {
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username": username, "password": password}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "login failed for {}", username);
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
async fn test_get_categories_returns_seeded_tree() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .uri("/api/knowledge/categories")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let arr = body.as_array().expect("array");
    assert!(arr.len() >= 5, "seed should produce at least 5 categories");
}

#[tokio::test]
async fn test_create_category_curator_role_allowed() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "curator", "Scholar2024!").await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/knowledge/categories")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({"name":"New Category","parent_id":"cat-root"}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_category_reviewer_role_forbidden() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/knowledge/categories")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(json!({"name":"Forbidden"}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_merge_cycle_returns_409() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "curator", "Scholar2024!").await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/knowledge/categories/merge")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({"source_id":"cat-mathematics","target_id":"cat-algebra"}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_bulk_preview_returns_conflicts() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "curator", "Scholar2024!").await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/knowledge/points/bulk/preview")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({
                "ids": ["kp-001","kp-002","kp-003"],
                "changes": { "difficulty": 1 }
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let conflicts = body.as_array().unwrap();
    // Seeded kps have difficulty 3, 4, 2 — all differ from target=1, so 3 conflicts.
    assert_eq!(conflicts.len(), 3);
}

#[tokio::test]
async fn test_bulk_apply_oversize_returns_400() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "curator", "Scholar2024!").await;

    let huge: Vec<String> = (0..1001).map(|i| format!("kp-{i}")).collect();
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/knowledge/points/bulk/apply")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({"ids": huge, "changes": {"difficulty": 4}}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
