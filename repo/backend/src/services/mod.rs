//! Service layer — business logic that owns SQLite reads/writes and audit logging.
//! Each module is implemented in its corresponding phase per PLAN.md.

pub mod auth_service;
pub mod knowledge_service;
pub mod question_service;
pub mod outcome_service;
pub mod store_service;
pub mod analytics_service;
pub mod file_service;
pub mod backup_service;
pub mod backup_scheduler;
pub mod audit_service;
pub mod abuse;
