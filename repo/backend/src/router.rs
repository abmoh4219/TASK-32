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
    routing::{get, post, put},
    Json, Router,
};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers::analytics as analytics_handlers;
use crate::handlers::auth as auth_handlers;
use crate::handlers::backup as backup_handlers;
use crate::handlers::knowledge as knowledge_handlers;
use crate::handlers::outcomes as outcome_handlers;
use crate::handlers::store as store_handlers;
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
        // Knowledge — categories
        .route(
            "/api/knowledge/categories",
            get(knowledge_handlers::list_categories).post(knowledge_handlers::create_category),
        )
        .route(
            "/api/knowledge/categories/tree",
            get(knowledge_handlers::get_tree),
        )
        .route(
            "/api/knowledge/categories/:id",
            put(knowledge_handlers::update_category)
                .delete(knowledge_handlers::delete_category),
        )
        .route(
            "/api/knowledge/categories/:id/references",
            get(knowledge_handlers::category_reference_count),
        )
        .route(
            "/api/knowledge/categories/merge",
            post(knowledge_handlers::merge_categories),
        )
        // Knowledge points
        .route(
            "/api/knowledge/points",
            get(knowledge_handlers::list_knowledge_points)
                .post(knowledge_handlers::create_knowledge_point),
        )
        .route(
            "/api/knowledge/points/:id",
            put(knowledge_handlers::update_knowledge_point)
                .delete(knowledge_handlers::delete_knowledge_point),
        )
        .route(
            "/api/knowledge/points/bulk/preview",
            post(knowledge_handlers::bulk_preview),
        )
        .route(
            "/api/knowledge/points/bulk/apply",
            post(knowledge_handlers::bulk_apply),
        )
        // Questions
        .route(
            "/api/knowledge/questions",
            get(knowledge_handlers::list_questions)
                .post(knowledge_handlers::create_question),
        )
        .route(
            "/api/knowledge/questions/:id",
            put(knowledge_handlers::update_question)
                .delete(knowledge_handlers::delete_question),
        )
        .route(
            "/api/knowledge/questions/:id/link",
            post(knowledge_handlers::link_question),
        )
        // Outcomes
        .route(
            "/api/outcomes",
            get(outcome_handlers::list_outcomes).post(outcome_handlers::create_outcome),
        )
        .route("/api/outcomes/:id", get(outcome_handlers::get_outcome))
        .route(
            "/api/outcomes/:id/contributors",
            post(outcome_handlers::add_contributor),
        )
        .route(
            "/api/outcomes/:id/contributors/:cid",
            axum::routing::delete(outcome_handlers::remove_contributor),
        )
        .route(
            "/api/outcomes/:id/submit",
            post(outcome_handlers::submit_outcome),
        )
        .route(
            "/api/outcomes/:id/approve",
            post(outcome_handlers::approve_outcome),
        )
        .route(
            "/api/outcomes/:id/reject",
            post(outcome_handlers::reject_outcome),
        )
        .route(
            "/api/outcomes/:id/evidence",
            post(outcome_handlers::upload_evidence),
        )
        .route(
            "/api/outcomes/:id/compare/:other_id",
            get(outcome_handlers::compare_outcomes),
        )
        // Store
        .route(
            "/api/store/products",
            get(store_handlers::list_products).post(store_handlers::create_product),
        )
        .route(
            "/api/store/promotions",
            get(store_handlers::list_promotions).post(store_handlers::create_promotion),
        )
        .route(
            "/api/store/promotions/:id/deactivate",
            post(store_handlers::deactivate_promotion),
        )
        .route(
            "/api/store/checkout",
            post(store_handlers::checkout),
        )
        .route(
            "/api/store/checkout/preview",
            post(store_handlers::preview_checkout),
        )
        .route("/api/store/orders", get(store_handlers::list_orders))
        .route("/api/store/orders/:id", get(store_handlers::get_order))
        // Analytics
        .route("/api/analytics/members", get(analytics_handlers::members))
        .route("/api/analytics/churn", get(analytics_handlers::churn))
        .route("/api/analytics/events", get(analytics_handlers::events))
        .route("/api/analytics/funds", get(analytics_handlers::fund_summary))
        .route(
            "/api/analytics/approval-cycles",
            get(analytics_handlers::approval_cycles),
        )
        .route(
            "/api/analytics/export/csv",
            post(analytics_handlers::export_csv),
        )
        .route(
            "/api/analytics/export/pdf",
            post(analytics_handlers::export_pdf),
        )
        .route(
            "/api/analytics/reports/schedule",
            post(analytics_handlers::schedule_report),
        )
        .route(
            "/api/analytics/reports",
            get(analytics_handlers::list_reports),
        )
        .route(
            "/api/analytics/reports/:id/download/:token",
            get(analytics_handlers::download_report),
        )
        // Backup
        .route(
            "/api/backup/history",
            get(backup_handlers::list_history),
        )
        .route("/api/backup/run", post(backup_handlers::run_backup))
        .route(
            "/api/backup/:id/restore-sandbox",
            post(backup_handlers::restore_sandbox),
        )
        .route("/api/backup/:id/activate", post(backup_handlers::activate))
        .route(
            "/api/backup/lifecycle-cleanup",
            post(backup_handlers::cleanup),
        )
        .route(
            "/api/backup/policy",
            get(backup_handlers::get_policy).put(backup_handlers::update_policy),
        )
        // Admin user management + audit log
        .route(
            "/api/admin/users",
            get(auth_handlers::admin_list_users).post(auth_handlers::admin_create_user),
        )
        .route(
            "/api/admin/users/:id/role",
            post(auth_handlers::admin_change_role),
        )
        .route(
            "/api/admin/users/:id/active",
            post(auth_handlers::admin_set_active),
        )
        .route("/api/admin/audit", get(auth_handlers::admin_audit_log))
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
