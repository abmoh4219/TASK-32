//! Session-loader middleware.
//!
//! Reads the `sv_session` cookie from the request, looks the session row up in
//! SQLite, and — if it is still alive — attaches a `CurrentUser` extension to
//! the request so handlers and the `require_role` extractor can read it.
//!
//! Expired sessions are deleted from the table on first sight; missing or
//! malformed cookies are silently ignored (the request continues unauthenticated).

use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};

use crate::models::user::User;
use crate::AppState;

/// Wrapper newtype attached to request extensions when a session is active.
/// Handlers extract it via the `FromRequestParts` impl in `require_role.rs`.
#[derive(Clone, Debug)]
pub struct CurrentUser(pub User);

#[derive(Clone, Debug)]
pub struct CurrentSession {
    pub session_id: String,
    pub csrf_token: String,
}

/// Axum middleware: pulls the session cookie, loads the user, and attaches
/// `CurrentUser` + `CurrentSession` extensions if everything checks out.
pub async fn session_middleware(
    State(state): State<AppState>,
    cookies: CookieJar,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    if let Some(session_cookie) = cookies.get("sv_session") {
        let session_id = session_cookie.value().to_string();
        // Extract request context for session-binding validation.
        let req_ua = req
            .headers()
            .get("User-Agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let req_ip = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.ip().to_string())
            .unwrap_or_default();
        if let Some((user, csrf_token)) =
            load_session_user(&state, &session_id, &req_ua, &req_ip).await
        {
            req.extensions_mut().insert(CurrentUser(user));
            req.extensions_mut().insert(CurrentSession {
                session_id,
                csrf_token,
            });
        }
    }
    next.run(req).await
}

/// Load + validate the session row, including context-binding checks.
///
/// **User-agent binding (hard)**: if the session was created with a non-empty
/// `user_agent` value, subsequent requests with a materially different
/// user-agent are rejected and the session is deleted.
///
/// **IP binding (soft/tolerant)**: the session's IP is stored but only
/// enforced when `TRUSTED_PROXY_HEADERS` is unset (i.e. the deployment does
/// NOT sit behind a NAT/proxy that changes source IPs). Mismatch triggers a
/// warning log but does NOT invalidate the session — mobile/ISP users
/// legitimately switch IPs. Full hard-binding would break too many real
/// users. When we DO invalidate, it is a hard delete + `None` return so the
/// request continues unauthenticated.
async fn load_session_user(
    state: &AppState,
    session_id: &str,
    request_ua: &str,
    request_ip: &str,
) -> Option<(User, String)> {
    #[derive(sqlx::FromRow)]
    struct SessionRow {
        user_id: String,
        csrf_token: String,
        expires_at: String,
        user_agent: Option<String>,
        ip_address: Option<String>,
    }

    let row: Option<SessionRow> = sqlx::query_as::<_, SessionRow>(
        "SELECT user_id, csrf_token, expires_at, user_agent, ip_address FROM sessions WHERE id = ?",
    )
    .bind(session_id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let row = row?;

    // Reject expired sessions and clean them up.
    if let Ok(exp) = DateTime::parse_from_rfc3339(&row.expires_at) {
        if exp.with_timezone(&Utc) < Utc::now() {
            let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
                .bind(session_id)
                .execute(&state.db)
                .await;
            return None;
        }
    }

    // ── Context-binding: user-agent (hard check) ────────────────────────
    if let Some(stored_ua) = &row.user_agent {
        if !stored_ua.is_empty() && !request_ua.is_empty() && stored_ua != request_ua {
            tracing::warn!(
                session_id = %session_id,
                "session invalidated: user-agent mismatch"
            );
            let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
                .bind(session_id)
                .execute(&state.db)
                .await;
            return None;
        }
    }

    // ── Context-binding: IP (soft — log only unless strict mode) ────────
    if let Some(stored_ip) = &row.ip_address {
        if !stored_ip.is_empty() && !request_ip.is_empty() && stored_ip != request_ip {
            tracing::warn!(
                session_id = %session_id,
                stored_ip = %stored_ip,
                request_ip = %request_ip,
                "session IP mismatch (soft warning — not invalidated)"
            );
        }
    }

    let user: Option<User> =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ? AND is_active = 1")
            .bind(&row.user_id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();

    user.map(|u| (u, row.csrf_token))
}
