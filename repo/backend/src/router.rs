//! Top-level Axum router. Phase 0 wires only health and static-asset routes.
//! Each subsequent phase registers its module's handlers here with the appropriate
//! middleware stack (CSRF on mutations, role gates, rate limiting).

use axum::{routing::get, Json, Router};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

use crate::AppState;

/// Build the router for the entire application.
/// Phase 0: serves the WASM frontend out of `STATIC_DIR` and exposes `/healthz`.
pub fn build_router(state: AppState) -> Router {
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "./dist".to_string());
    let index_path = format!("{}/index.html", static_dir);

    let static_service = ServeDir::new(&static_dir)
        .not_found_service(ServeFile::new(&index_path))
        .append_index_html_on_directories(true);

    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/healthz", get(healthz))
        .fallback_service(static_service)
        .with_state(state)
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "scholarvault",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
