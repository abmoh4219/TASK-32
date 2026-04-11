//! Audit log row mapping. Mirrors `audit_logs` exactly. There is intentionally no
//! `updated_at` column — the table is append-only by design and the `AuditService`
//! impl block contains only `log()` and `compute_hash()`, never `update` or `delete`.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}
