//! Frontend mask helper unit tests — pure functions, no DOM/WASM required.

use frontend::logic::mask::mask_last4;

#[test]
fn test_mask_phone_shows_last_4() {
    assert_eq!(mask_last4("9876543210"), "******3210");
}

#[test]
fn test_mask_id_shows_last_4() {
    assert_eq!(mask_last4("ID-4221-9988"), "********9988");
}

#[test]
fn test_mask_short_value_all_stars() {
    assert_eq!(mask_last4("123"), "***");
    assert_eq!(mask_last4(""), "");
}

#[test]
fn test_mask_exactly_4_chars_unchanged() {
    // 4 characters or fewer are entirely starred (otherwise we would leak the
    // whole value).
    assert_eq!(mask_last4("ABCD"), "****");
}

#[test]
fn test_mask_handles_unicode_chars() {
    // Mask operates on chars, not bytes — multibyte characters must not panic.
    assert_eq!(mask_last4("日本語ABCD"), "***ABCD");
}
