//! SQLx data models. Each struct corresponds to a row in a SQLite table and
//! derives `FromRow` so service-layer queries can return them directly.
//! Bodies land in Phase 1 once the migration files exist.

pub mod user;
pub mod knowledge;
pub mod outcome;
pub mod store;
pub mod analytics;
pub mod audit;
pub mod backup;
