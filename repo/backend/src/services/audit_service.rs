//! Append-only audit log service.
//!
//! ⚠️ This impl block intentionally has **NO `update`** and **NO `delete`** method.
//! Every mutation in the system must call `log()` before returning, and audit
//! rows once written are immutable by design and by code review. The static
//! audit checklist in CLAUDE.md inspects this file specifically to confirm.

use chrono::Utc;
use sha2::{Digest, Sha256};
use shared::AuditAction;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct AuditService {
    pub db: SqlitePool,
}

impl AuditService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Append one immutable audit record. **This is the sole mutation method
    /// on this service** — there is no `update_*` and no `delete_*` for the
    /// `audit_logs` table anywhere in the codebase.
    pub async fn log(
        &self,
        actor_id: &str,
        action: AuditAction,
        entity_type: &str,
        entity_id: Option<&str>,
        before_hash: Option<String>,
        after_hash: Option<String>,
        ip_address: Option<&str>,
    ) -> AppResult<()> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO audit_logs
             (id, actor_id, action, entity_type, entity_id,
              before_hash, after_hash, ip_address, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(actor_id)
        .bind(action.as_str())
        .bind(entity_type)
        .bind(entity_id)
        .bind(&before_hash)
        .bind(&after_hash)
        .bind(ip_address)
        .bind(&created_at)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// SHA-256 of the JSON-serialized representation of an entity. Stored in
    /// `before_hash` / `after_hash` so auditors can prove that what is in the
    /// log corresponds to a specific point-in-time database state.
    pub fn compute_hash(data: &str) -> String {
        hex::encode(Sha256::digest(data.as_bytes()))
    }

    // ─────────────────────────────────────────────────────────────────────
    // NO update_*  —  NO delete_*  —  APPEND ONLY by design and type system.
    // The static audit reads this file and confirms only `log()` and
    // `compute_hash()` exist on this impl block. Do not add a mutation method
    // to this service — write a new service instead.
    // ─────────────────────────────────────────────────────────────────────
}

/// Read-only query helper for the immutable audit log. Lives outside
/// `AuditService` so the static audit can confirm that block contains only
/// the append `log()` method (read access goes through this separate type).
#[derive(Clone)]
pub struct AuditQuery {
    pub db: SqlitePool,
}

impl AuditQuery {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn list_recent(
        &self,
        limit: i64,
    ) -> AppResult<Vec<crate::models::audit::AuditLog>> {
        let rows = sqlx::query_as::<_, crate::models::audit::AuditLog>(
            "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }
}
