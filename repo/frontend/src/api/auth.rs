//! Auth API client — login, logout, current user lookup.

use serde::{Deserialize, Serialize};

use crate::api::client::{get_json, post_json, ApiError};

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoginResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub full_name: Option<String>,
    pub csrf_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MeResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub csrf_token: String,
}

pub async fn login(username: &str, password: &str) -> Result<LoginResponse, ApiError> {
    let body = LoginRequest {
        username: username.to_string(),
        password: password.to_string(),
    };
    post_json::<_, LoginResponse>("/api/auth/login", &body).await
}

pub async fn logout() -> Result<serde_json::Value, ApiError> {
    post_json::<_, serde_json::Value>("/api/auth/logout", &serde_json::json!({})).await
}

pub async fn me() -> Result<MeResponse, ApiError> {
    get_json::<MeResponse>("/api/auth/me").await
}
