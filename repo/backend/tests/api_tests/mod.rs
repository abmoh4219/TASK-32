//! Backend API integration test entry point. Each submodule spins up an in-memory
//! Axum router with a temporary SQLite database and exercises real HTTP endpoints.

mod common;

mod auth_api;
mod knowledge_api;
mod outcome_api;
mod store_api;
mod analytics_api;
mod backup_api;
