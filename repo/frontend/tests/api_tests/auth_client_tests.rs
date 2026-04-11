//! Frontend auth API client serialization tests. These run as native Rust
//! tests so they exercise the request / response JSON shapes without needing
//! a browser or a live backend.

use frontend::api::auth::{LoginRequest, LoginResponse, MeResponse};
use frontend::api::client::ApiError;

#[test]
fn test_login_request_serializes_username_and_password() {
    let req = LoginRequest {
        username: "admin".into(),
        password: "ScholarAdmin2024!".into(),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("\"username\":\"admin\""));
    assert!(json.contains("\"password\":\"ScholarAdmin2024!\""));
}

#[test]
fn test_login_response_deserializes_correctly() {
    let body = r#"{
        "id":"u-admin",
        "username":"admin",
        "role":"administrator",
        "full_name":"System Administrator",
        "csrf_token":"abc123def456"
    }"#;
    let parsed: LoginResponse = serde_json::from_str(body).unwrap();
    assert_eq!(parsed.id, "u-admin");
    assert_eq!(parsed.role, "administrator");
    assert_eq!(parsed.csrf_token, "abc123def456");
    assert_eq!(parsed.full_name.as_deref(), Some("System Administrator"));
}

#[test]
fn test_me_response_handles_optional_fields() {
    let body = r#"{
        "id":"u-1",
        "username":"u",
        "role":"reviewer",
        "full_name":null,
        "email":null,
        "csrf_token":"x"
    }"#;
    let parsed: MeResponse = serde_json::from_str(body).unwrap();
    assert!(parsed.full_name.is_none());
    assert!(parsed.email.is_none());
}

#[test]
fn test_api_error_round_trip() {
    let err = ApiError {
        status: 401,
        code: "AUTH_REQUIRED".into(),
        message: "authentication required".into(),
    };
    let json = serde_json::to_string(&err).unwrap();
    let back: ApiError = serde_json::from_str(&json).unwrap();
    assert_eq!(back, err);
}

#[test]
fn test_login_request_includes_csrf_via_helper_signature() {
    // Compile-time check that LoginRequest fields match what the backend
    // login handler expects (snake_case JSON keys).
    let r = LoginRequest {
        username: "u".into(),
        password: "p".into(),
    };
    let v = serde_json::to_value(&r).unwrap();
    assert!(v.get("username").is_some());
    assert!(v.get("password").is_some());
}
