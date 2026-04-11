//! Argon2id password hashing.
//!
//! This module is the **only** place in the codebase that hashes or verifies a
//! password. The audit checklist in CLAUDE.md requires explicit Argon2 calls — not
//! just a `Cargo.toml` dependency — so the hashing flow lives here in plain
//! readable Rust:
//!
//! 1. Generate a fresh random salt with `OsRng` for every new password.
//! 2. Run Argon2id (the `Argon2::default()` configuration) over the password +
//!    salt and emit a self-describing PHC string that embeds the algorithm,
//!    parameters, salt, and digest.
//! 3. Verification re-parses that PHC string and lets the argon2 crate apply the
//!    embedded parameters — no plaintext comparison happens in this codebase.

use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};

use crate::error::{AppError, AppResult};

/// Hash a password using Argon2id with a randomly generated salt.
/// The output string embeds the salt and algorithm params for self-contained verification.
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("argon2 hash failed: {e}")))?
        .to_string();
    Ok(hash)
}

/// Verify a plain password against its Argon2id PHC hash. Returns Ok(true) on
/// match, Ok(false) on mismatch, and Err only if the stored hash itself is
/// malformed (corrupted database row).
pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("argon2 hash parse failed: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}
