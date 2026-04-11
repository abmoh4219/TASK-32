//! User table row mapping. Sensitive PII columns hold AES-256-GCM ciphertext;
//! they are decrypted on demand by the auth/admin services and masked for UI display.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: i64,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone_encrypted: Option<String>,
    pub national_id_encrypted: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoginAttempt {
    pub id: String,
    pub username: String,
    pub ip_address: Option<String>,
    pub attempted_at: String,
    pub success: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub csrf_token: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: String,
    pub created_at: String,
}
