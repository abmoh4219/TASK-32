//! Pure-function display masking — used by table cells to show only the last
//! four characters of a sensitive value. Mirrors `backend/security/encryption::mask_sensitive`.

/// Returns the value with everything but the last 4 chars replaced by `*`.
/// Operates on chars (not bytes) so non-ASCII strings are length-correct.
pub fn mask_last4(value: &str) -> String {
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
