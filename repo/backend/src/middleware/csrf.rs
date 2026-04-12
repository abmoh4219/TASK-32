//! CSRF middleware.
//!
//! For every state-changing HTTP verb (POST/PUT/PATCH/DELETE) the request must
//! include an `X-CSRF-Token` header that exactly matches the `csrf_token` cookie
//! issued at login. Comparison uses `constant_time_eq::constant_time_eq` to
//! prevent timing-based attacks against the token.
//!
//! Safe verbs (GET/HEAD/OPTIONS) and the `/api/auth/login` bootstrap endpoint
//! are skipped — login is the request that *issues* the cookie, so it cannot
//! demand it as input.

use axum::{
    body::Body,
    extract::{Request, State},
    http::Method,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;

use crate::error::{AppError, AppResult};
use crate::AppState;

/// Axum middleware function — wire up via
/// `axum::middleware::from_fn_with_state(state, csrf_middleware)`.
///
/// On state-changing verbs we enforce three invariants:
///   1. An `X-CSRF-Token` header is present.
///   2. It matches the `csrf_token` cookie (double-submit defence).
///   3. When a live session exists, it also matches the token stored on the
///      **session row in SQLite** — this is the session-bound check. An
///      attacker who steals a stale cookie but can't observe the current
///      session's stored token (e.g. after a refresh-csrf rotation) will
///      have the cookie and server-side record disagree.
pub async fn csrf_middleware(
    State(state): State<AppState>,
    cookies: CookieJar,
    req: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    if matches!(
        req.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    ) {
        return Ok(next.run(req).await);
    }

    // Login is exempt — it is the request that issues the csrf_token cookie.
    if req.uri().path() == "/api/auth/login" {
        return Ok(next.run(req).await);
    }

    let header_token = req
        .headers()
        .get("X-CSRF-Token")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::CsrfMissing)?
        .to_string();

    let cookie_token = cookies
        .get("csrf_token")
        .map(|c| c.value().to_string())
        .ok_or(AppError::CsrfMissing)?;

    // Constant-time comparison — never short-circuit on the first byte mismatch.
    if !constant_time_eq::constant_time_eq(header_token.as_bytes(), cookie_token.as_bytes()) {
        return Err(AppError::CsrfInvalid);
    }

    // Session-bound check: if a session cookie is present, the supplied header
    // must additionally match the token persisted on the `sessions` row.
    if let Some(session_cookie) = cookies.get("sv_session") {
        let session_id = session_cookie.value();
        let stored: Option<(String,)> = sqlx::query_as(
            "SELECT csrf_token FROM sessions WHERE id = ?",
        )
        .bind(session_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("csrf session lookup: {e}")))?;
        if let Some((session_token,)) = stored {
            if !constant_time_eq::constant_time_eq(
                header_token.as_bytes(),
                session_token.as_bytes(),
            ) {
                return Err(AppError::CsrfInvalid);
            }
        }
    }

    Ok(next.run(req).await)
}
