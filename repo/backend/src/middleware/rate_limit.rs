//! Rate-limiting middleware.
//!
//! Backed by the `governor` crate's keyed rate limiter. The key is the
//! authenticated `user_id` when present, otherwise the client IP — so the limit
//! is "per actor" rather than per process. Quota defaults to **60 requests per
//! minute** as required by SPEC.md, and a 429 response includes a `Retry-After`
//! header so callers can back off cleanly.

use std::num::NonZeroU32;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
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
        let n = NonZeroU32::new(per_minute).unwrap_or_else(|| NonZeroU32::new(60).unwrap());
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

/// Axum middleware: 60 requests per minute keyed by user_id (falls back to IP).
/// On overflow returns HTTP 429 with `Retry-After: 60`.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Choose the limiter key: user id when authenticated, else client IP, else 'anon'.
    let key = if let Some(CurrentUser(user)) = req.extensions().get::<CurrentUser>() {
        format!("user:{}", user.id)
    } else if let Some(ip) = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
    {
        format!("ip:{}", ip)
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
