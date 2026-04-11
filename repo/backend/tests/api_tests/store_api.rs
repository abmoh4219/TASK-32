//! Store HTTP integration tests — checkout flow + role gating + promo create.

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
async fn test_full_checkout_flow_applies_best_offer() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "store", "Scholar2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);

    // Cart of two seeded products.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/store/checkout")
        .header("content-type", "application/json")
        .header("cookie", cookie)
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({
                "items": [
                    {"product_id":"prod-book-1","product_name":"Linear Algebra Textbook","quantity":2,"unit_price":39.99},
                    {"product_id":"prod-book-2","product_name":"Physics Workbook","quantity":1,"unit_price":24.50}
                ]
            }).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 256 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let total = body["order"]["total"].as_f64().unwrap();
    let subtotal = body["order"]["subtotal"].as_f64().unwrap();
    let discount = body["order"]["discount_applied"].as_f64().unwrap();
    assert!(subtotal > 0.0);
    assert!(discount > 0.0, "seed promotion should give a discount");
    assert!((total - (subtotal - discount)).abs() < 1e-6);
    // Best offer trace contains a promotion name on each line.
    let line_items = body["result"]["line_items"].as_array().unwrap();
    assert!(line_items
        .iter()
        .all(|l| !l["promotion_applied"].is_null()));
}

#[tokio::test]
async fn test_checkout_no_eligible_promotions_zero_discount() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "store", "Scholar2024!").await;
    // Deactivate both seeded promos via the deactivate route.
    for promo_id in ["promo-spring-10", "promo-bundle-5"] {
        let req = Request::builder()
            .method(Method::POST)
            .uri(format!("/api/store/promotions/{}/deactivate", promo_id))
            .header("cookie", format!("{}; csrf_token={}", session, csrf))
            .header("X-CSRF-Token", csrf.clone())
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/store/checkout")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({"items":[{"product_id":"prod-book-1","product_name":"Linear Algebra Textbook","quantity":1,"unit_price":39.99}]}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["order"]["discount_applied"].as_f64().unwrap(), 0.0);
}

#[tokio::test]
async fn test_store_manager_creates_promotion() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "store", "Scholar2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/store/promotions")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({
                "name":"Test Promo",
                "description":"",
                "discount_value":15.0,
                "discount_type":"percent",
                "effective_from":"2026-04-01T00:00:00Z",
                "effective_until":"2099-12-31T23:59:59Z",
                "mutual_exclusion_group":null,
                "priority":3
            }).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_order_requires_auth_and_blocks_cross_user_access() {
    // Regression: /api/store/orders/:id previously had no auth and no ownership
    // check, allowing anyone who guessed an order id to read it. This test
    // proves (a) anonymous callers get 401, (b) another non-privileged user
    // gets 403, and (c) the owner and the store manager both succeed.
    let (app, _state) = setup_test_app().await;

    // 1. reviewer checks out — becomes the order owner.
    let (r_sess, r_csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let r_cookie = format!("{}; csrf_token={}", r_sess, r_csrf);
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/store/checkout")
        .header("content-type", "application/json")
        .header("cookie", r_cookie.clone())
        .header("X-CSRF-Token", r_csrf.clone())
        .body(Body::from(
            json!({
                "items": [
                    {"product_id":"prod-book-1","product_name":"Linear Algebra Textbook","quantity":1,"unit_price":39.99}
                ]
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 128 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let order_id = body["order"]["id"].as_str().unwrap().to_string();

    // 2. Anonymous request → 401.
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/store/orders/{}", order_id))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 3. A different, non-privileged user (finance) must NOT read this order.
    let (f_sess, f_csrf) = login_as(app.clone(), "finance", "Scholar2024!").await;
    let f_cookie = format!("{}; csrf_token={}", f_sess, f_csrf);
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/store/orders/{}", order_id))
        .header("cookie", f_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // 4. The owner (reviewer) can still read their own order.
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/store/orders/{}", order_id))
        .header("cookie", r_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 5. Store manager has privileged read access on any order.
    let (s_sess, s_csrf) = login_as(app.clone(), "store", "Scholar2024!").await;
    let s_cookie = format!("{}; csrf_token={}", s_sess, s_csrf);
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/store/orders/{}", order_id))
        .header("cookie", s_cookie)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_preview_checkout_requires_auth() {
    // Regression: preview_checkout previously had no extractor, now aligned
    // with checkout/list/get.
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "store", "Scholar2024!").await;

    // Anonymous call with a forged matching csrf cookie+header but no session
    // cookie must be rejected by the auth extractor (not just by CSRF).
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/store/checkout/preview")
        .header("content-type", "application/json")
        .header("cookie", "csrf_token=forged")
        .header("X-CSRF-Token", "forged")
        .body(Body::from(
            json!({"items":[{"product_id":"prod-book-1","product_name":"X","quantity":1,"unit_price":10.0}]}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Authenticated call → 200.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/store/checkout/preview")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({"items":[{"product_id":"prod-book-1","product_name":"X","quantity":1,"unit_price":10.0}]}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_reviewer_cannot_create_promotion() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/store/promotions")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({
                "name":"Forbidden",
                "description":"",
                "discount_value":1.0,
                "discount_type":"percent",
                "effective_from":"2026-04-01T00:00:00Z",
                "effective_until":"2099-12-31T23:59:59Z",
                "mutual_exclusion_group":null,
                "priority":1
            }).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
