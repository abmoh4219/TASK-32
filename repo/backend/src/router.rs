//! Top-level Axum router. Phase 2 wires:
//!   • Auth handlers (`/api/auth/*`) — login, logout, me, refresh-csrf
//!   • Session loading middleware (reads cookie, attaches CurrentUser)
//!   • CSRF middleware on every state-changing API request
//!   • Rate limit middleware (60 req/min per user) on `/api/**`
//!   • Security response headers on every response
//!   • Static file serving for the WASM frontend out of `STATIC_DIR`
//!
//! Each later phase appends its module's routes to `api_router` here.

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers::auth as auth_handlers;
use crate::middleware::csrf::csrf_middleware;
use crate::middleware::rate_limit::rate_limit_middleware;
use crate::middleware::security_headers::security_headers_middleware;
use crate::middleware::session::session_middleware;
use crate::AppState;

/// Build the router for the entire application.
pub fn build_router(state: AppState) -> Router {
    let api_routes = Router::new()
        // Auth — open to anonymous
        .route("/api/auth/login", post(auth_handlers::login))
        .route("/api/auth/logout", post(auth_handlers::logout))
        .route("/api/auth/me", get(auth_handlers::me))
        .route("/api/auth/refresh-csrf", post(auth_handlers::refresh_csrf))
        // Health
        .route("/api/healthz", get(healthz))
        // Layered: CSRF on mutations, then rate limit, then session loader.
        // Order matters: outermost (last `.layer`) runs first.
        .layer(axum_middleware::from_fn(csrf_middleware))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            session_middleware,
        ));

    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "./dist".to_string());
    let index_path = format!("{}/index.html", static_dir);
    let static_service = ServeDir::new(&static_dir)
        .not_found_service(ServeFile::new(&index_path))
        .append_index_html_on_directories(true);

    Router::new()
        .route("/healthz", get(healthz))
        .merge(api_routes)
        .fallback_service(static_service)
        .layer(axum_middleware::from_fn(security_headers_middleware))
        .with_state(state)
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "scholarvault",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
