//! Lightweight anti-abuse tracker for repeated invalid knowledge searches.
//!
//! A "search" is counted as **invalid** when the caller passes real filter
//! criteria but the query returns zero rows — that is the cheap scan pattern
//! the CLAUDE.md Open Questions call out (`3+ invalid searches → exponential
//! delay 2^n seconds`). The counter is keyed per actor (user id when
//! authenticated, falling back to the client IP) and decays on any valid
//! result so honest users never hit the backoff.
//!
//! The implementation is intentionally in-memory (DashMap) so there is no
//! extra infrastructure; it is a single shared value held inside `AppState`.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use std::sync::Arc;

use crate::error::AppError;

#[derive(Clone, Default)]
pub struct InvalidSearchTracker {
    inner: Arc<DashMap<String, Entry>>,
}

#[derive(Clone, Copy, Default)]
struct Entry {
    strikes: u32,
    blocked_until: Option<DateTime<Utc>>,
}

impl InvalidSearchTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reject the request if the actor is currently in backoff. The delay
    /// formula mirrors the CLAUDE.md note: `2^(strikes - 2)` seconds, capped.
    pub fn check(&self, actor: &str) -> Result<(), AppError> {
        if let Some(e) = self.inner.get(actor) {
            if let Some(until) = e.blocked_until {
                if until > Utc::now() {
                    return Err(AppError::RateLimit);
                }
            }
        }
        Ok(())
    }

    /// Record one invalid search. After 3 strikes the next attempt is blocked
    /// for `2^(strikes-2)` seconds (cap 300s).
    pub fn record_invalid(&self, actor: &str) {
        let mut e = self.inner.entry(actor.to_string()).or_default();
        e.strikes = e.strikes.saturating_add(1);
        if e.strikes >= 3 {
            let exp = (e.strikes - 2).min(8) as i64;
            let secs = (1i64 << exp).min(300);
            e.blocked_until = Some(Utc::now() + Duration::seconds(secs));
        }
    }

    /// Any valid (non-empty) result clears the strike counter.
    pub fn reset(&self, actor: &str) {
        self.inner.remove(actor);
    }
}
