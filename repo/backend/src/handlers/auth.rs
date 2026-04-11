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

/// Whether cookies should carry the `Secure` attribute. Safe-by-default:
/// enabled unless the deployment has explicitly opted into an HTTP dev path by
/// setting `APP_ENV=dev` (or `COOKIE_SECURE=false`). In any production-like
/// environment `main.rs` hard-fails at startup unless TLS is either terminated
/// upstream (`TRUSTED_TLS_PROXY=true`) or `COOKIE_SECURE` is explicitly set,
/// so by the time this runs the choice has already been validated.
fn cookies_secure() -> bool {
    if let Ok(raw) = std::env::var("COOKIE_SECURE") {
        return matches!(
            raw.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        );
    }
    // If APP_ENV is explicitly dev/local/test, stay compatible with HTTP dev.
    let dev = std::env::var("APP_ENV")
        .map(|v| {
            matches!(
                v.to_ascii_lowercase().as_str(),
                "dev" | "development" | "local" | "test"
            )
        })
        .unwrap_or(false);
    !dev
}

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

    let secure = cookies_secure();
    let session_cookie = Cookie::build(("sv_session", outcome.session_id.clone()))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    let csrf_cookie = Cookie::build(("csrf_token", outcome.csrf_token.clone()))
        .http_only(false)
        .secure(secure)
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

// ─── Admin user management ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
    pub full_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub role: String,
    pub is_active: i64,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
}

impl From<crate::models::user::User> for UserSummary {
    fn from(u: crate::models::user::User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            role: u.role,
            is_active: u.is_active,
            full_name: u.full_name,
            email: u.email,
            created_at: u.created_at,
        }
    }
}

pub async fn admin_list_users(
    State(state): State<AppState>,
    _admin: crate::middleware::require_role::RequireAdmin,
) -> AppResult<Json<Vec<UserSummary>>> {
    let svc = AuthService::new(state.db.clone());
    let users = svc.list_users().await?;
    Ok(Json(users.into_iter().map(UserSummary::from).collect()))
}

pub async fn admin_create_user(
    State(state): State<AppState>,
    crate::middleware::require_role::RequireAdmin(actor): crate::middleware::require_role::RequireAdmin,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<UserSummary>> {
    let svc = AuthService::new(state.db.clone());
    let user = svc
        .create_user(
            &req.username,
            &req.password,
            &req.role,
            req.full_name.as_deref(),
            req.email.as_deref(),
        )
        .await?;
    AuditService::new(state.db.clone())
        .log(
            &actor.id,
            AuditAction::Create,
            "user",
            Some(&user.id),
            None,
            Some(AuditService::compute_hash(&user.username)),
            None,
        )
        .await?;
    Ok(Json(UserSummary::from(user)))
}

#[derive(Deserialize)]
pub struct ChangeRoleRequest {
    pub role: String,
}

pub async fn admin_change_role(
    State(state): State<AppState>,
    crate::middleware::require_role::RequireAdmin(actor): crate::middleware::require_role::RequireAdmin,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<ChangeRoleRequest>,
) -> AppResult<Json<UserSummary>> {
    let svc = AuthService::new(state.db.clone());
    let user = svc.change_role(&id, &req.role).await?;
    AuditService::new(state.db.clone())
        .log(
            &actor.id,
            AuditAction::RoleChange,
            "user",
            Some(&id),
            None,
            Some(AuditService::compute_hash(&req.role)),
            None,
        )
        .await?;
    Ok(Json(UserSummary::from(user)))
}

#[derive(Deserialize)]
pub struct ActiveRequest {
    pub active: bool,
}

pub async fn admin_set_active(
    State(state): State<AppState>,
    crate::middleware::require_role::RequireAdmin(actor): crate::middleware::require_role::RequireAdmin,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<ActiveRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let svc = AuthService::new(state.db.clone());
    svc.set_active(&id, req.active).await?;
    AuditService::new(state.db.clone())
        .log(
            &actor.id,
            AuditAction::Update,
            "user",
            Some(&id),
            None,
            Some(AuditService::compute_hash(&format!("active={}", req.active))),
            None,
        )
        .await?;
    Ok(Json(serde_json::json!({"success": true})))
}

pub async fn admin_audit_log(
    State(state): State<AppState>,
    _admin: crate::middleware::require_role::RequireAdmin,
) -> AppResult<Json<Vec<crate::models::audit::AuditLog>>> {
    let q = crate::services::audit_service::AuditQuery::new(state.db.clone());
    Ok(Json(q.list_recent(200).await?))
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
        .secure(cookies_secure())
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    let cookies = cookies.add(csrf_cookie);
    Ok((cookies, Json(serde_json::json!({"csrf_token": new_token}))))
}
