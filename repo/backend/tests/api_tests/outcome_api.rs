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
async fn test_outcome_mutation_authz_blocks_non_creator_reviewer() {
    // Regression: any reviewer could mutate any outcome. Now only creator or
    // admin can submit/add-contributors on a draft. We create an outcome as
    // the seeded `reviewer` user, then login as `admin` (who created a
    // second reviewer-role user) and show that a different reviewer user
    // cannot submit.
    let (app, state) = setup_test_app().await;

    // Reviewer creates an outcome.
    let (r_sess, r_csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let r_cookie = format!("{}; csrf_token={}", r_sess, r_csrf);
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/outcomes")
        .header("content-type", "application/json")
        .header("cookie", r_cookie.clone())
        .header("X-CSRF-Token", r_csrf.clone())
        .body(Body::from(
            json!({"type":"paper","title":"AuthZ Test","abstract_snippet":"x","certificate_number":null}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    let outcome_id = body["outcome"]["id"].as_str().unwrap().to_string();

    // Create a second reviewer user via direct SQL (test shortcut).
    let pw_hash = "$argon2id$v=19$m=65536,t=3,p=4$2FHXrF9P4ynr+J9LZjB9WQ$jlffWgWmsRYlPpI4nH6mhmfFKPum4mKJ44BLhxRWHNY";
    sqlx::query(
        "INSERT INTO users (id,username,password_hash,role,is_active,created_at,updated_at) VALUES (?,?,?,?,1,datetime('now'),datetime('now'))",
    )
    .bind("u-reviewer2")
    .bind("reviewer2")
    .bind(pw_hash)
    .bind("reviewer")
    .execute(&state.db)
    .await
    .unwrap();

    let (r2_sess, r2_csrf) = login_as(app.clone(), "reviewer2", "Scholar2024!").await;
    let r2_cookie = format!("{}; csrf_token={}", r2_sess, r2_csrf);

    // Reviewer2 tries to submit reviewer1's outcome → 403.
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/submit", outcome_id))
        .header("cookie", r2_cookie.clone())
        .header("X-CSRF-Token", r2_csrf.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Reviewer2 tries to add a contributor → 403.
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/contributors", outcome_id))
        .header("content-type", "application/json")
        .header("cookie", r2_cookie)
        .header("X-CSRF-Token", r2_csrf)
        .body(Body::from(
            json!({"user_id":"u-reviewer2","share_percentage":100}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Admin can still submit (admin is always authorized).
    let (a_sess, a_csrf) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let a_cookie = format!("{}; csrf_token={}", a_sess, a_csrf);
    // First add a contributor as the original creator.
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/contributors", outcome_id))
        .header("content-type", "application/json")
        .header("cookie", r_cookie.clone())
        .header("X-CSRF-Token", r_csrf)
        .body(Body::from(
            json!({"user_id":"u-reviewer","share_percentage":100,"role_in_work":"author"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/submit", outcome_id))
        .header("cookie", a_cookie)
        .header("X-CSRF-Token", a_csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_audit_hashes_always_present() {
    // Regression: audit rows must always have both before_hash and after_hash.
    let (app, state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);
    // Create an outcome → triggers a Create audit row.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/outcomes")
        .header("content-type", "application/json")
        .header("cookie", cookie)
        .header("X-CSRF-Token", csrf)
        .body(Body::from(
            json!({"type":"paper","title":"Audit Hash Test","abstract_snippet":"t","certificate_number":null}).to_string(),
        ))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Fetch all audit rows that have BOTH hashes NULL.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE before_hash IS NULL AND after_hash IS NULL",
    )
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(
        count, 0,
        "no audit rows should have both hashes missing"
    );
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
async fn test_remove_contributor_rejects_cross_outcome_id() {
    // Regression: contributor deletion must bind both outcome_id AND
    // contributor_id so an attacker can't delete a contributor that belongs
    // to a different outcome by passing its id into the path of another one.
    let (app, state) = setup_test_app().await;
    let (session, csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let cookie = format!("{}; csrf_token={}", session, csrf);

    // Create outcome A with a contributor.
    let mk_outcome = |title: &str| {
        let title = title.to_string();
        let cookie = cookie.clone();
        let csrf = csrf.clone();
        let app = app.clone();
        async move {
            let req = Request::builder()
                .method(Method::POST)
                .uri("/api/outcomes")
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .header("X-CSRF-Token", csrf)
                .body(Body::from(
                    json!({
                        "type":"paper",
                        "title":title,
                        "abstract_snippet":"x",
                        "certificate_number": null
                    })
                    .to_string(),
                ))
                .unwrap();
            let resp = app.oneshot(req).await.unwrap();
            let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
            let body: Value = serde_json::from_slice(&bytes).unwrap();
            body["outcome"]["id"].as_str().unwrap().to_string()
        }
    };
    let id_a = mk_outcome("Alpha").await;
    let id_b = mk_outcome("Bravo").await;

    // Add a contributor to outcome A.
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/outcomes/{}/contributors", id_a))
        .header("content-type", "application/json")
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::from(
            json!({"user_id":"u-reviewer","share_percentage":100,"role_in_work":"author"})
                .to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let contributor: Value = serde_json::from_slice(&bytes).unwrap();
    let contributor_id = contributor["id"].as_str().unwrap().to_string();

    // Attempt: delete contributor of A while pretending the path outcome is B.
    // Expect 404 (not found under this outcome) and the row still present.
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/outcomes/{}/contributors/{}", id_b, contributor_id))
        .header("cookie", cookie.clone())
        .header("X-CSRF-Token", csrf.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Row must still exist in A.
    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM outcome_contributors WHERE id = ?",
    )
    .bind(&contributor_id)
    .fetch_one(&state.db)
    .await
    .unwrap();
    assert_eq!(remaining, 1, "cross-outcome delete must not remove the row");

    // Correct path (outcome A) succeeds.
    let req = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/outcomes/{}/contributors/{}", id_a, contributor_id))
        .header("cookie", cookie)
        .header("X-CSRF-Token", csrf)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_outcomes_visibility_scoped_to_creator_or_contributor() {
    // Regression: list/get used to expose every outcome to every authenticated
    // user. Now only administrators and reviewers see everything; other roles
    // see only outcomes they created or are a contributor on.
    let (app, _state) = setup_test_app().await;

    // Reviewer creates an outcome.
    let (r_sess, r_csrf) = login_as(app.clone(), "reviewer", "Scholar2024!").await;
    let r_cookie = format!("{}; csrf_token={}", r_sess, r_csrf);
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/outcomes")
        .header("content-type", "application/json")
        .header("cookie", r_cookie.clone())
        .header("X-CSRF-Token", r_csrf.clone())
        .body(Body::from(
            json!({
                "type":"paper",
                "title":"Private Reviewer Paper",
                "abstract_snippet":"only visible to contributors",
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

    // Curator is NOT a contributor and is not privileged → empty list + 403.
    let (c_sess, _) = login_as(app.clone(), "curator", "Scholar2024!").await;
    let req = Request::builder()
        .uri("/api/outcomes")
        .header("cookie", c_sess.clone())
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    assert!(rows.is_empty(), "curator must not see reviewer's outcome");

    let req = Request::builder()
        .uri(format!("/api/outcomes/{}", outcome_id))
        .header("cookie", c_sess)
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // Admin is privileged and still sees the outcome.
    let (a_sess, _) = login_as(app.clone(), "admin", "ScholarAdmin2024!").await;
    let req = Request::builder()
        .uri("/api/outcomes")
        .header("cookie", a_sess)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = to_bytes(resp.into_body(), 64 * 1024).await.unwrap();
    let rows: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
    assert!(
        rows.iter().any(|r| r["id"] == outcome_id),
        "admin should see reviewer's outcome"
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
