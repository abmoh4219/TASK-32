//! Cryptographic primitives used by ScholarVault.
//!
//! - `password`: Argon2id hashing and verification for user credentials
//! - `encryption`: AES-256-GCM field encryption + UI masking helpers
//! - `csrf`: random CSRF token generation
//!
//! Each submodule contains the explicit, audit-friendly implementation referenced
//! by the static code audit checklist in CLAUDE.md.

pub mod password;
pub mod encryption;
pub mod csrf;
