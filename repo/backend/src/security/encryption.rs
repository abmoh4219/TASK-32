//! AES-256-GCM field-level encryption and the matching UI masking helper.
//!
//! Used for sensitive PII columns (`phone`, `national_id`) so the database row
//! never holds plaintext. Each call generates a fresh random nonce — encrypting
//! the same plaintext twice produces two different ciphertexts.
//!
//! On the UI side `mask_sensitive` returns the value with everything but the last
//! four characters replaced by `*`, matching the SPEC requirement for last-4
//! disclosure of sensitive fields.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

use crate::error::{AppError, AppResult};

const NONCE_LEN: usize = 12;

/// Encrypt a sensitive field with AES-256-GCM. Output format is
/// `base64(nonce[12 bytes] || ciphertext)` so a single text column round-trips
/// the value end-to-end.
pub fn encrypt_field(plaintext: &str, key: &[u8; 32]) -> AppResult<String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| AppError::Internal("aes-gcm encrypt failed".to_string()))?;
    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(nonce.as_slice());
    combined.extend_from_slice(&ciphertext);
    Ok(BASE64.encode(combined))
}

/// Decrypt a value previously produced by `encrypt_field`. Splits the leading
/// 12-byte nonce off the buffer and runs AES-256-GCM in reverse with the same key.
pub fn decrypt_field(encoded: &str, key: &[u8; 32]) -> AppResult<String> {
    let data = BASE64
        .decode(encoded)
        .map_err(|e| AppError::Internal(format!("base64 decode: {e}")))?;
    if data.len() < NONCE_LEN {
        return Err(AppError::Internal("ciphertext too short".to_string()));
    }
    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| AppError::Internal("aes-gcm decrypt failed".to_string()))?;
    String::from_utf8(plaintext).map_err(|e| AppError::Internal(format!("utf8: {e}")))
}

/// Encrypt an arbitrary byte slice with AES-256-GCM. Output is the raw
/// `nonce[12] || ciphertext` byte sequence (no base64), suitable for binary
/// storage on disk such as backup bundles or encrypted evidence files.
pub fn encrypt_bytes(plaintext: &[u8], key: &[u8; 32]) -> AppResult<Vec<u8>> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| AppError::Internal("aes-gcm encrypt failed".to_string()))?;
    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(nonce.as_slice());
    combined.extend_from_slice(&ciphertext);
    Ok(combined)
}

/// Reverse of `encrypt_bytes` — strips the nonce and runs AES-256-GCM in reverse.
pub fn decrypt_bytes(blob: &[u8], key: &[u8; 32]) -> AppResult<Vec<u8>> {
    if blob.len() < NONCE_LEN {
        return Err(AppError::Internal("ciphertext too short".to_string()));
    }
    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| AppError::Internal("aes-gcm decrypt failed".to_string()))
}

/// Mask a sensitive value for UI display — show only the last 4 characters,
/// replace the rest with `*`. Operates on chars (not bytes) to handle non-ASCII
/// values safely. Example: `"9876543210"` → `"******3210"`.
pub fn mask_sensitive(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    let n = chars.len();
    if n == 0 {
        return String::new();
    }
    if n <= 4 {
        return "*".repeat(n);
    }
    let mut out = String::with_capacity(n);
    for _ in 0..(n - 4) {
        out.push('*');
    }
    for c in &chars[n - 4..] {
        out.push(*c);
    }
    out
}
