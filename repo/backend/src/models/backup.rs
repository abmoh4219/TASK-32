//! Backup record + retention policy row mappings.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BackupRecord {
    pub id: String,
    pub backup_type: String,
    pub bundle_path: String,
    pub sha256_hash: String,
    pub status: String,
    pub size_bytes: i64,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub restored_at: Option<String>,
    /// `database` or `files`. NULL for legacy rows created before the split.
    pub artifact_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RetentionPolicy {
    pub id: String,
    pub daily_retention: i64,
    pub monthly_retention: i64,
    pub preserve_financial: i64,
    pub preserve_ip: i64,
    pub updated_at: String,
    pub updated_by: Option<String>,
}
