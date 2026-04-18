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
async fn test_admin_create_user_encrypts_and_masks_pii() {
    // Regression: phone + national_id must be AES-encrypted before hitting
    // SQLite and only ever returned masked (last 4) to the UI.
    let (app, state) = setup_test_app().await;

    // Admin login.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"admin","password":"ScholarAdmin2024!"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
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

    let phone = "5551234567";
    let national_id = "A9876543210";
    let payload = json!({
        "username":"pii-user",
        "password":"Scholar2024!",
        "role":"content_curator",
        "phone": phone,
        "national_id": national_id
    })
    .to_string();
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(payload))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let body_text = body.to_string();

    // Masked presentation only — plaintext must NOT appear anywhere in the
    // response envelope.
    assert!(!body_text.contains(phone), "phone plaintext leaked in response");
    assert!(
        !body_text.contains(national_id),
        "national_id plaintext leaked in response"
    );
    assert_eq!(body["phone_masked"], "******4567");
    assert_eq!(body["national_id_masked"], "*******3210");

    // DB row must contain ciphertext, not plaintext.
    let row: (Option<String>, Option<String>) = sqlx::query_as(
        "SELECT phone_encrypted, national_id_encrypted FROM users WHERE username = 'pii-user'",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    let phone_ct = row.0.expect("phone_encrypted must be populated");
    let nid_ct = row.1.expect("national_id_encrypted must be populated");
    assert_ne!(phone_ct, phone, "stored phone must not be plaintext");
    assert_ne!(nid_ct, national_id, "stored national_id must not be plaintext");
    // AES-256-GCM + 12-byte nonce + base64 → at least ~24 chars even for a
    // short plaintext. Sanity check that we're storing an encrypted blob.
    assert!(phone_ct.len() > phone.len() + 8);
    assert!(nid_ct.len() > national_id.len() + 8);
}

#[tokio::test]
async fn test_csrf_session_bound_rejects_stale_cookie() {
    // Double-submit alone is not enough: tamper the `sessions.csrf_token`
    // column directly so the cookie no longer matches the server-side record
    // and confirm the mutation path now 403s.
    let (app, state) = setup_test_app().await;
    let (session, csrf) = login_as_admin(app.clone()).await;

    // Rotate server-side csrf_token for this session out-of-band so it no
    // longer matches the cookie the client is still sending.
    let session_id = session.strip_prefix("sv_session=").unwrap();
    sqlx::query("UPDATE sessions SET csrf_token = 'rotated-server-side' WHERE id = ?")
        .bind(session_id)
        .execute(&state.db)
        .await
        .unwrap();

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/refresh-csrf")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "session-bound CSRF must reject a cookie that no longer matches the session row"
    );
}

#[tokio::test]
async fn test_security_headers_present_on_representative_routes() {
    let (app, _state) = setup_test_app().await;
    for path in ["/healthz", "/api/healthz"] {
        let req = Request::builder().uri(path).body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let headers = resp.headers();
        assert_eq!(resp.status(), StatusCode::OK, "{} must 200", path);
        let hsts = headers
            .get("Strict-Transport-Security")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(hsts.contains("max-age="), "HSTS missing on {}", path);
        let csp = headers
            .get("Content-Security-Policy")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default();
        assert!(csp.contains("default-src"), "CSP missing on {}", path);
        assert_eq!(
            headers.get("X-Frame-Options").and_then(|v| v.to_str().ok()),
            Some("DENY"),
            "X-Frame-Options missing on {}",
            path
        );
        assert_eq!(
            headers
                .get("X-Content-Type-Options")
                .and_then(|v| v.to_str().ok()),
            Some("nosniff"),
            "X-Content-Type-Options missing on {}",
            path
        );
    }
}

#[tokio::test]
async fn test_internal_errors_do_not_leak_raw_details() {
    // Issue the middleware lookup against an unknown path parameter that
    // forces a DB-backed not-found; assert the response body does not contain
    // SQL/SQLx noise from the raw Database/Internal error variants.
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as_admin(app.clone()).await;

    // Hit an admin-only path with a bogus id to provoke an internal lookup
    // path; most responses will already be generic, but we additionally force
    // a raw Database error by requesting report download with a garbage token.
    let req = Request::builder()
        .uri("/api/analytics/reports/does-not-exist/download/nope")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let text = String::from_utf8_lossy(&bytes).to_string();
    // Envelope must not carry SQLx/sqlite leakage. Use a few high-signal
    // substrings that commonly appear in raw errors from our stack.
    for needle in [
        "sqlx",
        "SELECT",
        "no such table",
        "FOREIGN KEY",
        "panicked",
    ] {
        assert!(
            !text.contains(needle),
            "response leaked raw error substring `{}`: {}",
            needle,
            text
        );
    }
    let _ = csrf;
}

/// Finding B regression: refresh_csrf must write an audit record with before/after hashes.
#[tokio::test]
async fn test_refresh_csrf_writes_audit_record() {
    let (app, state) = setup_test_app().await;
    let (session, csrf) = login_as_admin(app.clone()).await;

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/refresh-csrf")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Check that an audit row was created for the csrf_token rotation.
    let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT before_hash, after_hash FROM audit_logs WHERE entity_type = 'csrf_token' ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .unwrap();
    let (before, after) = row.expect("audit row for csrf_token must exist");
    assert!(before.is_some(), "before_hash must be populated");
    assert!(after.is_some(), "after_hash must be populated");
    assert_ne!(before, after, "before and after hashes should differ after rotation");
}

/// Finding B regression: every mutating endpoint must write both before_hash and after_hash.
#[tokio::test]
async fn test_audit_hashes_always_populated() {
    let (app, state) = setup_test_app().await;
    let (session, csrf) = login_as_admin(app.clone()).await;

    // Trigger a mutation — create a user.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({
                "username":"audit-test-user",
                "password":"Scholar2024!",
                "role":"reviewer"
            }).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // ALL audit rows should have non-null before_hash and after_hash.
    let nulls: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE before_hash IS NULL OR after_hash IS NULL",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(nulls, 0, "no audit rows should have NULL before_hash or after_hash");
}

/// Finding E regression: different usernames on same IP do not lock each other out.
#[tokio::test]
async fn test_lockout_does_not_cross_accounts_on_same_ip() {
    let (app, _state) = setup_test_app().await;

    // Fail 4 times for admin (just under lockout threshold).
    for _ in 0..4 {
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

    // A different user (curator) on the same IP should NOT be locked out.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"curator","password":"Scholar2024!"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK,
        "curator should NOT be locked out by admin's failures on the same IP");
}

async fn login_as_admin(app: axum::Router) -> (String, String) {
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"admin","password":"ScholarAdmin2024!"}).to_string(),
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
async fn test_seed_users_all_present() {
    let pool = setup_test_db().await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 5, "seed migration should have inserted 5 users");
}

// ─── GET /api/auth/me ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_me_authenticated_returns_user_info() {
    let (app, _state) = setup_test_app().await;
    let (session, _csrf) = login_as_admin(app.clone()).await;
    let req = Request::builder()
        .uri("/api/auth/me")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["username"], "admin");
    assert_eq!(body["role"], "administrator");
    assert!(body["csrf_token"].as_str().unwrap().len() >= 32);
}

#[tokio::test]
async fn test_me_anonymous_returns_401() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .uri("/api/auth/me")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ─── GET /api/admin/users ────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_list_users_returns_all_seeded() {
    let (app, _state) = setup_test_app().await;
    let (session, _csrf) = login_as_admin(app.clone()).await;
    let req = Request::builder()
        .uri("/api/admin/users")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let arr = body.as_array().expect("users must be array");
    assert!(arr.len() >= 5, "should return all seeded users");
}

#[tokio::test]
async fn test_admin_list_users_anonymous_returns_401() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .uri("/api/admin/users")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_list_users_non_admin_returns_403() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"curator","password":"Scholar2024!"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let mut session = String::new();
    for h in resp.headers().get_all("set-cookie").iter() {
        let v = h.to_str().unwrap_or("");
        if let Some(rest) = v.strip_prefix("sv_session=") {
            session = format!("sv_session={}", rest.split(';').next().unwrap());
        }
    }
    let req = Request::builder()
        .uri("/api/admin/users")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ─── POST /api/admin/users/:id/role ──────────────────────────────────────────

#[tokio::test]
async fn test_admin_change_role_succeeds() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as_admin(app.clone()).await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users/u-curator/role")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(json!({"role":"reviewer"}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_change_role_anonymous_returns_401() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users/u-curator/role")
        .header("content-type", "application/json")
        .header("cookie", "csrf_token=forged")
        .header("X-CSRF-Token", "forged")
        .body(Body::from(json!({"role":"reviewer"}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_change_role_non_admin_returns_403() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"curator","password":"Scholar2024!"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
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
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users/u-reviewer/role")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(json!({"role":"administrator"}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_admin_change_role_not_found_returns_404() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as_admin(app.clone()).await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users/nonexistent-user-id/role")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(json!({"role":"reviewer"}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ─── POST /api/admin/users/:id/active ────────────────────────────────────────

#[tokio::test]
async fn test_admin_set_active_deactivates_user() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as_admin(app.clone()).await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users/u-curator/active")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(json!({"active":false}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_set_active_anonymous_returns_401() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users/u-curator/active")
        .header("content-type", "application/json")
        .header("cookie", "csrf_token=forged")
        .header("X-CSRF-Token", "forged")
        .body(Body::from(json!({"active":false}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_set_active_non_admin_returns_403() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"curator","password":"Scholar2024!"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
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
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users/u-reviewer/active")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf)
        .body(Body::from(json!({"active":false}).to_string()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ─── GET /api/admin/audit ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_audit_log_returns_records() {
    let (app, _state) = setup_test_app().await;
    let (session, csrf) = login_as_admin(app.clone()).await;
    // Trigger a mutation to ensure at least one audit record exists.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/admin/users")
        .header("content-type", "application/json")
        .header("cookie", format!("{}; csrf_token={}", session, csrf))
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(
            json!({"username":"audit-log-test","password":"Scholar2024!","role":"reviewer"}).to_string(),
        ))
        .unwrap();
    let _ = app.clone().oneshot(req).await.unwrap();

    let req = Request::builder()
        .uri("/api/admin/audit")
        .header("cookie", session.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 128 * 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(body.as_array().map(|a| !a.is_empty()).unwrap_or(false), "audit log must contain records");
}

#[tokio::test]
async fn test_admin_audit_log_anonymous_returns_401() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .uri("/api/admin/audit")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_admin_audit_log_non_admin_returns_403() {
    let (app, _state) = setup_test_app().await;
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"username":"curator","password":"Scholar2024!"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let mut session = String::new();
    for h in resp.headers().get_all("set-cookie").iter() {
        let v = h.to_str().unwrap_or("");
        if let Some(rest) = v.strip_prefix("sv_session=") {
            session = format!("sv_session={}", rest.split(';').next().unwrap());
        }
    }
    let req = Request::builder()
        .uri("/api/admin/audit")
        .header("cookie", session)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
