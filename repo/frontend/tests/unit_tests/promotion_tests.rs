//! Frontend promotion display + datetime helper unit tests.

use frontend::logic::promotion::{
    datetime_local_to_iso, format_discount, format_total_savings, iso_to_mmddyyyy,
    mmddyyyy_12h_to_iso,
};

#[test]
fn test_discount_display_percent_type() {
    assert_eq!(format_discount("percent", 10.0), "10%");
    assert_eq!(format_discount("percent", 25.0), "25%");
}

#[test]
fn test_discount_display_fixed_type() {
    assert_eq!(format_discount("fixed", 5.0), "$5.00 off");
    assert_eq!(format_discount("fixed", 12.5), "$12.50 off");
}

#[test]
fn test_promotion_time_format_mm_dd_yyyy() {
    let formatted = iso_to_mmddyyyy("2026-04-15T13:30:00Z");
    // Format should be MM/DD/YYYY hh:mm AM/PM
    assert!(formatted.starts_with("04/15/2026"), "got: {}", formatted);
    assert!(formatted.contains("PM"), "should be 12-hour PM, got: {}", formatted);
}

#[test]
fn test_total_savings_calculation() {
    assert_eq!(format_total_savings(&[1.50, 2.50, 3.00]), "$7.00");
    assert_eq!(format_total_savings(&[]), "$0.00");
}

#[test]
fn test_datetime_local_to_iso_appends_seconds_and_z() {
    assert_eq!(datetime_local_to_iso("2026-04-15T13:30"), "2026-04-15T13:30:00Z");
}

#[test]
fn test_mmddyyyy_12h_to_iso_happy_path() {
    assert_eq!(
        mmddyyyy_12h_to_iso("04/15/2026", "01:30", "PM").unwrap(),
        "2026-04-15T13:30:00Z"
    );
    assert_eq!(
        mmddyyyy_12h_to_iso("04/15/2026", "12:00", "AM").unwrap(),
        "2026-04-15T00:00:00Z"
    );
    assert_eq!(
        mmddyyyy_12h_to_iso("12/31/2099", "12:00", "PM").unwrap(),
        "2099-12-31T12:00:00Z"
    );
}

#[test]
fn test_mmddyyyy_12h_to_iso_rejects_bad_input() {
    assert!(mmddyyyy_12h_to_iso("2026-04-15", "13:30", "PM").is_none());
    assert!(mmddyyyy_12h_to_iso("04/15/2026", "13:30", "PM").is_none()); // hour > 12
    assert!(mmddyyyy_12h_to_iso("04/15/2026", "01:30", "XX").is_none());
}
