//! Rate-limiting middleware.
//!
//! Backed by the `governor` crate's keyed rate limiter. The key is the
//! authenticated `user_id` when present, otherwise the client IP — so the limit
//! is "per actor" rather than per process. Quota defaults to **60 requests per
//! minute** as required by SPEC.md, and a 429 response includes a `Retry-After`
//! header so callers can back off cleanly.

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::DefaultClock,
    state::keyed::DefaultKeyedStateStore,
    Quota, RateLimiter,
};

use crate::middleware::session::CurrentUser;
use crate::AppState;

/// Shared rate-limit state. Cloned into `AppState` so middleware can read it via
/// `State<AppState>`.
#[derive(Clone)]
pub struct RateLimitState {
    limiter: Arc<RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>>,
}

impl RateLimitState {
    /// Create a new keyed rate limiter at `per_minute` requests / 60 seconds.
    /// Defaults to 60 if the supplied value is zero.
    pub fn new(per_minute: u32) -> Self {
        // SAFETY: 60 is a non-zero literal, so the inner constructor never returns None.
        let fallback = NonZeroU32::new(60).unwrap_or(NonZeroU32::MIN);
        let n = NonZeroU32::new(per_minute).unwrap_or(fallback);
        let quota = Quota::per_minute(n);
        Self {
            limiter: Arc::new(RateLimiter::keyed(quota)),
        }
    }

    /// Returns true when the request fits within the quota for `key`.
    pub fn check(&self, key: &str) -> bool {
        self.limiter.check_key(&key.to_string()).is_ok()
    }
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self::new(60)
    }
}

/// Returns true when the deployment has explicitly opted in to trusting
/// `X-Forwarded-For` (i.e. sits behind a known proxy). Without this signal the
/// middleware ignores caller-controlled headers and keys off the socket peer
/// to prevent simple header-spoofing abuse of the rate limiter.
pub fn trusted_proxy_headers() -> bool {
    std::env::var("TRUSTED_PROXY_HEADERS")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::trusted_proxy_headers;

    #[test]
    fn trusted_proxy_defaults_off() {
        std::env::remove_var("TRUSTED_PROXY_HEADERS");
        assert!(!trusted_proxy_headers());
    }

    #[test]
    fn trusted_proxy_opt_in_values() {
        for v in ["1", "true", "yes", "on", "TRUE"] {
            std::env::set_var("TRUSTED_PROXY_HEADERS", v);
            assert!(trusted_proxy_headers(), "expected opt-in for {}", v);
        }
        std::env::set_var("TRUSTED_PROXY_HEADERS", "no");
        assert!(!trusted_proxy_headers());
        std::env::remove_var("TRUSTED_PROXY_HEADERS");
    }
}

/// Axum middleware: 60 requests per minute keyed by user_id (falls back to
/// socket peer IP). `X-Forwarded-For` is only honoured when
/// `TRUSTED_PROXY_HEADERS=true` so untrusted callers can't spoof the key.
/// On overflow returns HTTP 429 with `Retry-After: 60`.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Choose the limiter key: authenticated user id → forwarded header (only
    // when explicitly trusted) → socket peer address → "anon".
    let key = if let Some(CurrentUser(user)) = req.extensions().get::<CurrentUser>() {
        format!("user:{}", user.id)
    } else if trusted_proxy_headers() {
        if let Some(ip) = req
            .headers()
            .get("X-Forwarded-For")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(',').next())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            format!("ip:{}", ip)
        } else if let Some(ConnectInfo(addr)) =
            req.extensions().get::<ConnectInfo<SocketAddr>>()
        {
            format!("peer:{}", addr.ip())
        } else {
            "anon".to_string()
        }
    } else if let Some(ConnectInfo(addr)) =
        req.extensions().get::<ConnectInfo<SocketAddr>>()
    {
        format!("peer:{}", addr.ip())
    } else {
        "anon".to_string()
    };

    if state.rate_limit.check(&key) {
        next.run(req).await
    } else {
        let mut response =
            (StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded").into_response();
        response
            .headers_mut()
            .insert("Retry-After", HeaderValue::from_static("60"));
        response
    }
}
