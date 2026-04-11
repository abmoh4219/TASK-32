//! CSRF token generator. Each session is paired with a 256-bit random token
//! issued at login and stored alongside the session row.

use rand::RngCore;

/// Generate a new CSRF token: 32 random bytes hex-encoded into a 64-char string.
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
