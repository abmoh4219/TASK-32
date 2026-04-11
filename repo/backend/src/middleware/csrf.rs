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
    extract::Request,
    http::Method,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;

use crate::error::{AppError, AppResult};

/// Axum middleware function — wire up via `axum::middleware::from_fn(csrf_middleware)`.
pub async fn csrf_middleware(
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

    Ok(next.run(req).await)
}
