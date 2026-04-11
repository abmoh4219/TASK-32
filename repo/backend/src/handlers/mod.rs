//! Axum HTTP handlers — thin adapters that delegate to the service layer.
//! Implementations land in their respective PLAN.md phases.

pub mod auth;
pub mod knowledge;
pub mod outcomes;
pub mod store;
pub mod analytics;
pub mod files;
pub mod backup;
