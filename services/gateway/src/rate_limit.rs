//! A deterministic fixed-window rate limiter.
//!
//! Each client key (client IP by default) gets a counter scoped to a fixed time
//! window. When the window rolls over, the counter resets. Because the current
//! time is supplied by the caller — an injected clock in the server, a literal
//! value in tests — the limiter is fully deterministic and requires no timers.

use std::collections::HashMap;
use std::sync::Mutex;

/// The decision returned for a single admission check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateDecision {
    /// The request is within quota; `remaining` requests are still allowed.
    Allowed { remaining: u32 },
    /// The quota is exhausted; the window resets at `retry_after` seconds.
    Limited { retry_after: u64 },
}

impl RateDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed { .. })
    }
}

/// Per-key state: the window this counter belongs to and how many hits it holds.
#[derive(Clone, Copy, Debug)]
struct Window {
    start: u64,
    count: u32,
}

/// A fixed-window limiter keyed by an opaque client identifier.
#[derive(Debug)]
pub struct RateLimiter {
    quota: u32,
    window_secs: u64,
    windows: Mutex<HashMap<String, Window>>,
}

impl RateLimiter {
    /// Builds a limiter allowing `quota` requests per `window_secs` seconds.
    pub fn new(quota: u32, window_secs: u64) -> Self {
        Self { quota, window_secs: window_secs.max(1), windows: Mutex::new(HashMap::new()) }
    }

    /// Records an attempt for `key` at time `now` and returns the decision.
    pub fn check(&self, key: &str, now: u64) -> RateDecision {
        let window_start = now - (now % self.window_secs);
        let mut windows = self.windows.lock().expect("rate limiter mutex poisoned");
        let entry =
            windows.entry(key.to_owned()).or_insert(Window { start: window_start, count: 0 });
        if entry.start != window_start {
            entry.start = window_start;
            entry.count = 0;
        }
        if entry.count >= self.quota {
            let retry_after = (window_start + self.window_secs).saturating_sub(now);
            return RateDecision::Limited { retry_after };
        }
        entry.count += 1;
        RateDecision::Allowed { remaining: self.quota - entry.count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_quota_then_limits() {
        let limiter = RateLimiter::new(2, 60);
        assert!(matches!(limiter.check("a", 0), RateDecision::Allowed { remaining: 1 }));
        assert!(matches!(limiter.check("a", 1), RateDecision::Allowed { remaining: 0 }));
        assert!(matches!(limiter.check("a", 2), RateDecision::Limited { .. }));
    }

    #[test]
    fn window_reset_restores_quota() {
        let limiter = RateLimiter::new(1, 60);
        assert!(limiter.check("a", 10).is_allowed());
        assert!(!limiter.check("a", 20).is_allowed());
        // Next window (>= 60s) resets the counter.
        assert!(limiter.check("a", 60).is_allowed());
    }

    #[test]
    fn keys_are_isolated() {
        let limiter = RateLimiter::new(1, 60);
        assert!(limiter.check("a", 0).is_allowed());
        assert!(limiter.check("b", 0).is_allowed());
    }

    #[test]
    fn retry_after_points_at_window_boundary() {
        let limiter = RateLimiter::new(1, 60);
        assert!(limiter.check("a", 10).is_allowed());
        match limiter.check("a", 10) {
            RateDecision::Limited { retry_after } => assert_eq!(retry_after, 50),
            other => panic!("expected limited, got {other:?}"),
        }
    }
}
