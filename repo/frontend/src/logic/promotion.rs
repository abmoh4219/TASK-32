//! Pure-function helpers for the promotion / checkout UI: discount display
//! formatting and MM/DD/YYYY 12-hour datetime conversion.

use chrono::{DateTime, NaiveDateTime};

/// Formatted display of a promotion's discount value (e.g. "10%" or "$5.00 off").
pub fn format_discount(discount_type: &str, value: f64) -> String {
    match discount_type {
        "percent" => format!("{:.0}%", value),
        "fixed" => format!("${:.2} off", value),
        _ => "—".to_string(),
    }
}

/// Convert an RFC 3339 / ISO 8601 timestamp to MM/DD/YYYY hh:mm AM/PM display.
pub fn iso_to_mmddyyyy(iso: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(iso) {
        return dt.format("%m/%d/%Y %I:%M %p").to_string();
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(iso, "%Y-%m-%dT%H:%M:%S") {
        return dt.format("%m/%d/%Y %I:%M %p").to_string();
    }
    iso.to_string()
}

/// Convert an HTML `datetime-local` value (`YYYY-MM-DDTHH:MM`) into an
/// RFC 3339 string suitable for posting to the backend.
pub fn datetime_local_to_iso(value: &str) -> String {
    if value.len() < 16 {
        return value.to_string();
    }
    format!("{}:00Z", &value[..16])
}

/// Total currency saved across a list of line items, formatted with two decimals.
/// Adding `+ 0.0` normalises any IEEE 754 negative zero into positive zero so
/// the formatted output is never `"$-0.00"`.
pub fn format_total_savings(per_line: &[f64]) -> String {
    let total: f64 = per_line.iter().sum::<f64>() + 0.0;
    format!("${:.2}", total)
}
