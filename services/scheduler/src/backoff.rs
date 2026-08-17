//! Deterministic retry backoff schedule for failed jobs.
//!
//! When a job fails retryably it is requeued; this module computes how long a
//! caller *should* wait before the next attempt. The schedule is a capped
//! exponential: `base * factor^(attempt - 1)`, clamped to `maximum`. It is a
//! pure, allocation-light calculation with saturating arithmetic, so it never
//! panics on large attempt counts and is fully reproducible. The scheduler
//! requeues eagerly today, so this delay is advisory metadata a dispatcher or
//! broker seam can honor.

use crate::SchedulerError;

/// A capped exponential backoff schedule expressed in milliseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Backoff {
    base_ms: u64,
    factor: u64,
    maximum_ms: u64,
}

impl Backoff {
    /// Builds a schedule, rejecting degenerate parameters.
    ///
    /// `base_ms` and `factor` must be positive and `maximum_ms` at least
    /// `base_ms`, otherwise the schedule could stall or shrink.
    pub fn new(base_ms: u64, factor: u64, maximum_ms: u64) -> Result<Self, SchedulerError> {
        if base_ms == 0 {
            return Err(SchedulerError::InvalidConfig("backoff base_ms must be positive".into()));
        }
        if factor == 0 {
            return Err(SchedulerError::InvalidConfig("backoff factor must be positive".into()));
        }
        if maximum_ms < base_ms {
            return Err(SchedulerError::InvalidConfig(
                "backoff maximum_ms must be at least base_ms".into(),
            ));
        }
        Ok(Self { base_ms, factor, maximum_ms })
    }

    /// The delay before `attempt` (1-based). Attempt 0 or 1 yields `base_ms`.
    ///
    /// Growth saturates rather than overflowing, and every value is capped at
    /// `maximum_ms`, so the schedule is monotonic non-decreasing and bounded.
    pub fn delay_ms(&self, attempt: u32) -> u64 {
        let steps = attempt.saturating_sub(1);
        let mut delay = self.base_ms;
        for _ in 0..steps {
            delay = delay.saturating_mul(self.factor);
            if delay >= self.maximum_ms {
                return self.maximum_ms;
            }
        }
        delay.min(self.maximum_ms)
    }

    /// The delays for attempts `1..=count`.
    pub fn schedule(&self, count: u32) -> Vec<u64> {
        (1..=count).map(|attempt| self.delay_ms(attempt)).collect()
    }
}

impl Default for Backoff {
    /// A conservative default: 100 ms base, doubling, capped at 30 s.
    fn default() -> Self {
        Self { base_ms: 100, factor: 2, maximum_ms: 30_000 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_attempt_is_the_base_delay() {
        let backoff = Backoff::new(100, 2, 10_000).unwrap();
        assert_eq!(backoff.delay_ms(1), 100);
        assert_eq!(backoff.delay_ms(0), 100);
    }

    #[test]
    fn delay_grows_exponentially_then_caps() {
        let backoff = Backoff::new(100, 2, 1_000).unwrap();
        assert_eq!(backoff.schedule(6), vec![100, 200, 400, 800, 1_000, 1_000]);
    }

    #[test]
    fn growth_saturates_without_panicking() {
        let backoff = Backoff::new(1, 1_000_000_000, u64::MAX).unwrap();
        assert_eq!(backoff.delay_ms(u32::MAX), u64::MAX);
    }

    #[test]
    fn rejects_degenerate_parameters() {
        assert!(Backoff::new(0, 2, 10).is_err());
        assert!(Backoff::new(10, 0, 10).is_err());
        assert!(Backoff::new(10, 2, 5).is_err());
    }
}
