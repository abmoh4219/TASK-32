//! Security response-headers middleware.
//!
//! Adds the standard hardening headers to every response so the deployed app is
//! protected against clickjacking, MIME sniffing, mixed content, and referrer
//! leakage even when no explicit handler-level configuration exists.

use axum::{body::Body, extract::Request, http::HeaderValue, middleware::Next, response::Response};

/// Axum middleware function — adds HSTS, CSP, X-Frame-Options, X-Content-Type-Options,
/// and Referrer-Policy headers. Wire up with `axum::middleware::from_fn(security_headers_middleware)`.
pub async fn security_headers_middleware(req: Request<Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
             connect-src 'self'; font-src 'self' data:;",
        ),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );

    response
}
