//! Pure-function validators used by the contributor allocation step + the
//! promotion date-range form. Tested directly in `frontend/tests/unit_tests/`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareTotalState {
    /// Shares total to exactly 100% — submit is allowed and the indicator goes green.
    Complete,
    /// Total is below 100 — partial allocation. UI shows the running total in red.
    Under,
    /// Total is above 100 — invalid. UI shows the overshoot in red.
    Over,
}

/// Reduce a list of contribution share percentages to a `ShareTotalState`.
pub fn share_total_state(shares: &[i64]) -> ShareTotalState {
    let total: i64 = shares.iter().sum();
    if total == 100 {
        ShareTotalState::Complete
    } else if total < 100 {
        ShareTotalState::Under
    } else {
        ShareTotalState::Over
    }
}

/// CSS color the share total bar should display.
pub fn share_total_color(shares: &[i64]) -> &'static str {
    match share_total_state(shares) {
        ShareTotalState::Complete => "#10B981", // green
        ShareTotalState::Under | ShareTotalState::Over => "#EF4444", // red
    }
}

/// True if `start` is strictly before `end`. Both must be in RFC3339 / ISO 8601.
pub fn is_valid_date_range(start: &str, end: &str) -> bool {
    match (
        chrono::DateTime::parse_from_rfc3339(start),
        chrono::DateTime::parse_from_rfc3339(end),
    ) {
        (Ok(s), Ok(e)) => s < e,
        _ => false,
    }
}
