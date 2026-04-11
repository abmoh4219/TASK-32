//! Outcome HTTP integration tests — full register flow + share validation +
//! duplicate fingerprint rejection.

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
async fn test_full_outcome_registration_flow() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);

    // Create outcome
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/outcomes")
        .header("content-type", "application/json")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(
            json!({
                "type":"paper",
                "title":"Integration Flow Paper",
                "abstract_snippet":"Tests the full register + submit flow",
                "certificate_number": null
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let outcome_id = body["outcome"]["id"].as_str().unwrap().to_string();

    // Add a 100% contributor
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/contributors", outcome_id))
        .header("content-type", "application/json")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(
            json!({"user_id":"u-reviewer","share_percentage":100,"role_in_work":"author"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Submit
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/submit", outcome_id))
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_submit_without_100_percent_returns_400() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/outcomes")
        .header("content-type", "application/json")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(
            json!({
                "type":"patent",
                "title":"Half Allocated Patent",
                "abstract_snippet":"only 50% allocated",
                "certificate_number": null
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let id = body["outcome"]["id"].as_str().unwrap().to_string();

    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/contributors", id))
        .header("content-type", "application/json")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(
            json!({"user_id":"u-reviewer","share_percentage":50}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/submit", id))
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_evidence_upload_duplicate_fingerprint_returns_409() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/outcomes")
        .header("content-type", "application/json")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(
            json!({"type":"paper","title":"With evidence","abstract_snippet":"x","certificate_number":null}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let outcome_id = serde_json::from_slice::<Value>(&bytes).unwrap()["outcome"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Build a real PDF byte payload via a multipart body string.
    let pdf: &[u8] = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\nrest";
    let boundary = "----test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"evidence.pdf\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: application/pdf\r\n\r\n");
    body.extend_from_slice(pdf);
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/evidence", outcome_id))
        .header(
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(body.clone()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "first upload must succeed");

    // Same bytes again → 409
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/evidence", outcome_id))
        .header(
            "content-type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf)
        .body(Body::from(body))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::CONFLICT,
        "duplicate upload should be rejected"
    );
}

#[tokio::test]
async fn test_outcomes_read_endpoints_reject_anonymous() {
    // Regression: list/get/compare used to be openly readable. Confirm they now
    // require an authenticated session.
    let (app, _state) = setup_test_app().await;

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/outcomes")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/outcomes/anything")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/outcomes/a/compare/b")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
