//! Frontend share-total + date-range validator unit tests.

use frontend::logic::validation::{
    is_valid_date_range, share_total_color, share_total_state, ShareTotalState,
};

#[test]
fn test_share_total_display_100_shows_green() {
    assert_eq!(share_total_state(&[60, 40]), ShareTotalState::Complete);
    assert_eq!(share_total_color(&[60, 40]), "#10B981");
}

#[test]
fn test_share_total_display_99_shows_red() {
    assert_eq!(share_total_state(&[50, 49]), ShareTotalState::Under);
    assert_eq!(share_total_color(&[50, 49]), "#EF4444");
}

#[test]
fn test_share_total_display_101_shows_red() {
    assert_eq!(share_total_state(&[60, 41]), ShareTotalState::Over);
    assert_eq!(share_total_color(&[60, 41]), "#EF4444");
}

#[test]
fn test_share_total_empty_is_under() {
    assert_eq!(share_total_state(&[]), ShareTotalState::Under);
}

#[test]
fn test_date_range_start_before_end_valid() {
    assert!(is_valid_date_range(
        "2026-04-01T00:00:00Z",
        "2026-04-30T23:59:59Z"
    ));
}

#[test]
fn test_date_range_start_after_end_invalid() {
    assert!(!is_valid_date_range(
        "2026-05-01T00:00:00Z",
        "2026-04-01T00:00:00Z"
    ));
}

#[test]
fn test_date_range_invalid_format_returns_false() {
    assert!(!is_valid_date_range("not-a-date", "also-not"));
}
