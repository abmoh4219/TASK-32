//! HTTP client modules wrapping `gloo_net::http` for talking to the Axum backend.
//! Each module mirrors a backend handler group. Implementations land alongside
//! their owning phase.

pub mod client;
pub mod auth;
pub mod knowledge;
pub mod outcomes;
pub mod store;
pub mod analytics;
pub mod backup;
