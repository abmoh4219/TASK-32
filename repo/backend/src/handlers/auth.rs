//! Authentication HTTP handlers.
//!
//! `/api/auth/login`   POST — username + password → session cookie + csrf cookie
//! `/api/auth/logout`  POST — clears the session cookie and removes the row
//! `/api/auth/me`      GET  — returns the current user (or 401 if not logged in)
//! `/api/auth/refresh-csrf` POST — issues a fresh CSRF token for the current session

use axum::{
    extract::{ConnectInfo, State},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::error::{AppError, AppResult};
use crate::middleware::session::{CurrentSession, CurrentUser};
use crate::services::audit_service::AuditService;
use crate::services::auth_service::AuthService;
use crate::AppState;
use shared::AuditAction;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub full_name: Option<String>,
    pub csrf_token: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub id: String,
    pub username: String,
    pub role: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub csrf_token: String,
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<AppState>,
    cookies: CookieJar,
    addr: Option<ConnectInfo<SocketAddr>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<(CookieJar, Json<LoginResponse>)> {
    if req.username.trim().is_empty() || req.password.is_empty() {
        return Err(AppError::Validation(
            "username and password are required".to_string(),
        ));
    }

    let ip = addr
        .map(|c| c.0.ip().to_string())
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let auth = AuthService::new(state.db.clone());
    let outcome = auth.login(&req.username, &req.password, &ip).await?;

    let session_cookie = Cookie::build(("sv_session", outcome.session_id.clone()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    let csrf_cookie = Cookie::build(("csrf_token", outcome.csrf_token.clone()))
        .http_only(false)
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    let cookies = cookies.add(session_cookie).add(csrf_cookie);

    let audit = AuditService::new(state.db.clone());
    let _ = audit
        .log(
            &outcome.user.id,
            AuditAction::Login,
            "user",
            Some(&outcome.user.id),
            None,
            None,
            Some(&ip),
        )
        .await;

    Ok((
        cookies,
        Json(LoginResponse {
            id: outcome.user.id,
            username: outcome.user.username,
            role: outcome.user.role,
            full_name: outcome.user.full_name,
            csrf_token: outcome.csrf_token,
        }),
    ))
}

/// POST /api/auth/logout — clears the session row and the cookies.
pub async fn logout(
    State(state): State<AppState>,
    cookies: CookieJar,
) -> AppResult<(CookieJar, Json<serde_json::Value>)> {
    if let Some(session_cookie) = cookies.get("sv_session") {
        let auth = AuthService::new(state.db.clone());
        auth.logout(session_cookie.value()).await?;
    }
    let cookies = cookies
        .remove(Cookie::from("sv_session"))
        .remove(Cookie::from("csrf_token"));
    Ok((cookies, Json(serde_json::json!({"success": true}))))
}

/// GET /api/auth/me — current user, requires an active session.
pub async fn me(
    user: Option<axum::extract::Extension<CurrentUser>>,
    session: Option<axum::extract::Extension<CurrentSession>>,
) -> AppResult<Json<MeResponse>> {
    let CurrentUser(user) = user.ok_or(AppError::Auth)?.0;
    let csrf_token = session
        .map(|s| s.0.csrf_token.clone())
        .unwrap_or_default();
    Ok(Json(MeResponse {
        id: user.id,
        username: user.username,
        role: user.role,
        full_name: user.full_name,
        email: user.email,
        csrf_token,
    }))
}

/// POST /api/auth/refresh-csrf — rotates the CSRF token for the current session.
pub async fn refresh_csrf(
    State(state): State<AppState>,
    cookies: CookieJar,
    session: Option<axum::extract::Extension<CurrentSession>>,
) -> AppResult<(CookieJar, Json<serde_json::Value>)> {
    let session = session.ok_or(AppError::Auth)?.0;
    let new_token = crate::security::csrf::generate_token();
    sqlx::query("UPDATE sessions SET csrf_token = ? WHERE id = ?")
        .bind(&new_token)
        .bind(&session.session_id)
        .execute(&state.db)
        .await?;
    let csrf_cookie = Cookie::build(("csrf_token", new_token.clone()))
        .http_only(false)
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    let cookies = cookies.add(csrf_cookie);
    Ok((cookies, Json(serde_json::json!({"csrf_token": new_token}))))
}
