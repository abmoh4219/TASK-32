//! ScholarVault backend library crate.
//!
//! Exposed as a library so integration tests in `tests/` can import services and
//! security helpers directly. The binary in `src/main.rs` simply calls into this
//! crate's `run()` to start the Axum server.

pub mod error;
pub mod db;
pub mod security;
pub mod middleware;
pub mod services;
pub mod handlers;
pub mod models;
pub mod router;

pub use error::{AppError, AppResult};

use std::sync::Arc;
use sqlx::SqlitePool;

use crate::middleware::rate_limit::RateLimitState;

/// Shared application state passed into every Axum handler via the `State` extractor.
#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub encryption_key: Arc<[u8; 32]>,
    pub signing_key: Arc<String>,
    pub rate_limit: RateLimitState,
}

/// Derive a 32-byte AES-256 key from an arbitrary-length string by truncating or
/// right-padding with zeroes. Used so the operator can configure the key as plain
/// text in `.env` while the cipher still receives a fixed-size byte array.
pub fn derive_key(material: &str) -> [u8; 32] {
    let bytes = material.as_bytes();
    let mut out = [0u8; 32];
    let n = bytes.len().min(32);
    out[..n].copy_from_slice(&bytes[..n]);
    out
}
