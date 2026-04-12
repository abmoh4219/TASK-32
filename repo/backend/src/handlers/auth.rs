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
    headers: axum::http::HeaderMap,
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
    let ua = headers
        .get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let auth = AuthService::new(state.db.clone());
    let outcome = auth
        .login(&req.username, &req.password, &ip, ua.as_deref())
        .await?;

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
            Some(AuditService::compute_hash("no_session")),
            Some(AuditService::compute_hash(&outcome.session_id)),
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

/// POST /api/auth/logout — clears the session row and the cookies. Emits an
/// audit log row so the actor + timestamp of every logout is recorded.
pub async fn logout(
    State(state): State<AppState>,
    cookies: CookieJar,
    user: Option<axum::extract::Extension<CurrentUser>>,
) -> AppResult<(CookieJar, Json<serde_json::Value>)> {
    if let Some(session_cookie) = cookies.get("sv_session") {
        let auth = AuthService::new(state.db.clone());
        auth.logout(session_cookie.value()).await?;
    }
    if let Some(u) = user {
        let actor_id = u.0 .0.id.clone();
        let session_id = cookies
            .get("sv_session")
            .map(|c| c.value().to_string())
            .unwrap_or_default();
        let _ = AuditService::new(state.db.clone())
            .log(
                &actor_id,
                AuditAction::Logout,
                "user",
                Some(&actor_id),
                Some(AuditService::compute_hash(&session_id)),
                Some(crate::services::audit_service::HASH_ENTITY_DELETED.to_string()),
                None,
            )
            .await;
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
    /// Plaintext PII accepted **only on the request**. Encrypted at the
    /// service layer before hitting SQLite — never stored or echoed raw.
    pub phone: Option<String>,
    pub national_id: Option<String>,
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
    /// Masked last-4 presentation of encrypted PII. `None` when the column is
    /// not set. The plaintext is *never* included in any API response.
    pub phone_masked: Option<String>,
    pub national_id_masked: Option<String>,
}

impl UserSummary {
    /// Build a response DTO from a database row. PII columns are decrypted
    /// with the supplied key and then masked for presentation — the UI never
    /// sees plaintext. Decryption failures are swallowed to avoid leaking
    /// ciphertext into the response; the field simply becomes `None`.
    pub fn from_user_masked(u: crate::models::user::User, key: &[u8; 32]) -> Self {
        let phone_masked = u
            .phone_encrypted
            .as_deref()
            .and_then(|ct| crate::security::encryption::decrypt_field(ct, key).ok())
            .map(|plain| crate::security::encryption::mask_sensitive(&plain));
        let national_id_masked = u
            .national_id_encrypted
            .as_deref()
            .and_then(|ct| crate::security::encryption::decrypt_field(ct, key).ok())
            .map(|plain| crate::security::encryption::mask_sensitive(&plain));
        Self {
            id: u.id,
            username: u.username,
            role: u.role,
            is_active: u.is_active,
            full_name: u.full_name,
            email: u.email,
            created_at: u.created_at,
            phone_masked,
            national_id_masked,
        }
    }
}

pub async fn admin_list_users(
    State(state): State<AppState>,
    _admin: crate::middleware::require_role::RequireAdmin,
) -> AppResult<Json<Vec<UserSummary>>> {
    let svc = AuthService::new(state.db.clone());
    let users = svc.list_users().await?;
    let key: &[u8; 32] = &state.encryption_key;
    Ok(Json(
        users
            .into_iter()
            .map(|u| UserSummary::from_user_masked(u, key))
            .collect(),
    ))
}

pub async fn admin_create_user(
    State(state): State<AppState>,
    crate::middleware::require_role::RequireAdmin(actor): crate::middleware::require_role::RequireAdmin,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<UserSummary>> {
    let svc = AuthService::new(state.db.clone());
    let key: [u8; 32] = *state.encryption_key;
    let user = svc
        .create_user(
            &req.username,
            &req.password,
            &req.role,
            req.full_name.as_deref(),
            req.email.as_deref(),
            req.phone.as_deref(),
            req.national_id.as_deref(),
            &key,
        )
        .await?;
    AuditService::new(state.db.clone())
        .log(
            &actor.id,
            AuditAction::Create,
            "user",
            Some(&user.id),
            Some(crate::services::audit_service::HASH_ENTITY_CREATED.to_string()),
            Some(AuditService::compute_hash(&user.username)),
            None,
        )
        .await?;
    Ok(Json(UserSummary::from_user_masked(user, &key)))
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
    // Capture before-state for audit.
    let before_user = svc.get_user(&id).await?;
    let before_hash = AuditService::compute_hash(&before_user.role);
    let user = svc.change_role(&id, &req.role).await?;
    AuditService::new(state.db.clone())
        .log(
            &actor.id,
            AuditAction::RoleChange,
            "user",
            Some(&id),
            Some(before_hash),
            Some(AuditService::compute_hash(&req.role)),
            None,
        )
        .await?;
    Ok(Json(UserSummary::from_user_masked(
        user,
        &state.encryption_key,
    )))
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
    let before_user = svc.get_user(&id).await?;
    let before_hash = AuditService::compute_hash(&format!("active={}", before_user.is_active == 1));
    svc.set_active(&id, req.active).await?;
    AuditService::new(state.db.clone())
        .log(
            &actor.id,
            AuditAction::Update,
            "user",
            Some(&id),
            Some(before_hash),
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
/// Audited as a security-state mutation with before/after hashes of the old and
/// new token values.
pub async fn refresh_csrf(
    State(state): State<AppState>,
    cookies: CookieJar,
    session: Option<axum::extract::Extension<CurrentSession>>,
    user: Option<axum::extract::Extension<CurrentUser>>,
) -> AppResult<(CookieJar, Json<serde_json::Value>)> {
    let session = session.ok_or(AppError::Auth)?.0;
    let old_token = session.csrf_token.clone();
    let new_token = crate::security::csrf::generate_token();
    sqlx::query("UPDATE sessions SET csrf_token = ? WHERE id = ?")
        .bind(&new_token)
        .bind(&session.session_id)
        .execute(&state.db)
        .await?;
    // Audit the CSRF rotation as a security-state change.
    if let Some(u) = user {
        let _ = AuditService::new(state.db.clone())
            .log(
                &u.0 .0.id,
                AuditAction::Update,
                "csrf_token",
                Some(&session.session_id),
                Some(AuditService::compute_hash(&old_token)),
                Some(AuditService::compute_hash(&new_token)),
                None,
            )
            .await;
    }
    let csrf_cookie = Cookie::build(("csrf_token", new_token.clone()))
        .http_only(false)
        .secure(cookies_secure())
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    let cookies = cookies.add(csrf_cookie);
    Ok((cookies, Json(serde_json::json!({"csrf_token": new_token}))))
}
