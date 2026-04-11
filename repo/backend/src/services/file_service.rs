//! Evidence file storage service.
//!
//! Performs offline validation of every uploaded file (magic-number + MIME +
//! 25 MB cap), refuses duplicate uploads via SHA-256 fingerprint, encrypts the
//! payload with AES-256-GCM and writes the ciphertext to the configured
//! evidence directory.

use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::outcome::EvidenceFile;
use crate::security::encryption;

pub const MAX_FILE_BYTES: usize = 25 * 1024 * 1024;
pub const ALLOWED_MIME_TYPES: &[&str] = &["application/pdf", "image/jpeg", "image/png"];

#[derive(Clone)]
pub struct FileService {
    pub db: SqlitePool,
    pub encryption_key: [u8; 32],
    pub storage_dir: PathBuf,
}

impl FileService {
    pub fn new(db: SqlitePool, encryption_key: [u8; 32], storage_dir: PathBuf) -> Self {
        Self {
            db,
            encryption_key,
            storage_dir,
        }
    }

    /// Validate uploaded file offline using magic-number inspection + MIME check.
    /// No external services — uses the `infer` crate to read header bytes.
    pub fn validate_file(bytes: &[u8], declared_mime: &str) -> AppResult<()> {
        if bytes.len() > MAX_FILE_BYTES {
            return Err(AppError::FileTooLarge {
                size: bytes.len(),
                max: MAX_FILE_BYTES,
            });
        }
        let detected = infer::get(bytes).ok_or(AppError::UnknownFileType)?;
        if !ALLOWED_MIME_TYPES.contains(&detected.mime_type()) {
            return Err(AppError::InvalidFileType(detected.mime_type().to_string()));
        }
        if detected.mime_type() != declared_mime {
            return Err(AppError::MimeMismatch {
                declared: declared_mime.to_string(),
                detected: detected.mime_type().to_string(),
            });
        }
        Ok(())
    }

    /// SHA-256 hex fingerprint of the **plaintext** bytes — used for dedup
    /// (so the same evidence file can't be re-uploaded under a new id).
    pub fn fingerprint(bytes: &[u8]) -> String {
        hex::encode(Sha256::digest(bytes))
    }

    /// Upload a piece of evidence: validate → fingerprint → reject duplicates →
    /// encrypt → write to disk → record metadata. Returns the new
    /// `EvidenceFile` row.
    pub async fn upload_evidence(
        &self,
        outcome_id: &str,
        bytes: &[u8],
        filename: &str,
        declared_mime: &str,
        uploader_id: &str,
    ) -> AppResult<EvidenceFile> {
        Self::validate_file(bytes, declared_mime)?;

        let fingerprint = Self::fingerprint(bytes);
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT id FROM evidence_files WHERE sha256_fingerprint = ?")
                .bind(&fingerprint)
                .fetch_optional(&self.db)
                .await?;
        if existing.is_some() {
            return Err(AppError::Conflict(format!(
                "evidence file already uploaded (fingerprint {})",
                &fingerprint[..16]
            )));
        }

        let ciphertext = encryption::encrypt_field(
            // base64-encoded plaintext as the inner string keeps the on-disk
            // representation as a single base64 blob and avoids any binary
            // issues with the encrypt_field interface.
            &base64::engine::general_purpose::STANDARD.encode(bytes),
            &self.encryption_key,
        )?;

        // Persist to disk under <storage_dir>/<outcome_id>/<file_id>
        let file_id = Uuid::new_v4().to_string();
        let dir = self.storage_dir.join(outcome_id);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(&file_id);
        std::fs::write(&path, ciphertext.as_bytes())?;

        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO evidence_files
             (id, outcome_id, filename, mime_type, stored_path, file_size, sha256_fingerprint, uploaded_by, uploaded_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&file_id)
        .bind(outcome_id)
        .bind(filename)
        .bind(declared_mime)
        .bind(path.to_string_lossy().to_string())
        .bind(bytes.len() as i64)
        .bind(&fingerprint)
        .bind(uploader_id)
        .bind(&now)
        .execute(&self.db)
        .await?;

        let row = sqlx::query_as::<_, EvidenceFile>("SELECT * FROM evidence_files WHERE id = ?")
            .bind(&file_id)
            .fetch_one(&self.db)
            .await?;
        Ok(row)
    }

    pub async fn list_for_outcome(&self, outcome_id: &str) -> AppResult<Vec<EvidenceFile>> {
        let rows = sqlx::query_as::<_, EvidenceFile>(
            "SELECT * FROM evidence_files WHERE outcome_id = ? ORDER BY uploaded_at DESC",
        )
        .bind(outcome_id)
        .fetch_all(&self.db)
        .await?;
        Ok(rows)
    }
}

use base64::Engine as _;
