//! Retry and dead-letter policy for failed jobs.
//!
//! When a worker reports a failure, the scheduler must decide whether to requeue
//! the job for another attempt or move it to the dead-letter terminal state. That
//! decision is exactly two inputs — whether the failure is retryable and whether
//! the attempt budget is exhausted — so it lives here as a small, total function
//! extracted from the scheduler core. The associated [`crate::backoff::Backoff`]
//! answers the companion question of *how long* to wait before the next attempt.

use crate::backoff::Backoff;

/// What to do with a job after a failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureAction {
    /// Requeue for another attempt.
    Requeue,
    /// Give up: move to the dead-letter state.
    DeadLetter,
}

/// The scheduler's retry/dead-letter policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    maximum_attempts: u32,
    backoff: Backoff,
}

impl RetryPolicy {
    /// Builds a policy with the given attempt budget and a default backoff.
    pub fn new(maximum_attempts: u32) -> Self {
        Self { maximum_attempts, backoff: Backoff::default() }
    }

    /// Overrides the advisory backoff schedule.
    pub fn with_backoff(mut self, backoff: Backoff) -> Self {
        self.backoff = backoff;
        self
    }

    pub fn maximum_attempts(&self) -> u32 {
        self.maximum_attempts
    }

    /// Decides the action for a failure at the given (current) attempt number.
    ///
    /// A retryable failure requeues only while attempts remain; a permanent
    /// failure or an exhausted budget dead-letters.
    pub fn on_failure(&self, retryable: bool, attempt: u32) -> FailureAction {
        if retryable && attempt < self.maximum_attempts {
            FailureAction::Requeue
        } else {
            FailureAction::DeadLetter
        }
    }

    /// The advisory delay before the next attempt following `attempt`.
    pub fn retry_delay_ms(&self, attempt: u32) -> u64 {
        self.backoff.delay_ms(attempt.saturating_add(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_failure_requeues_within_budget() {
        let policy = RetryPolicy::new(3);
        assert_eq!(policy.on_failure(true, 1), FailureAction::Requeue);
        assert_eq!(policy.on_failure(true, 2), FailureAction::Requeue);
    }

    #[test]
    fn exhausted_budget_dead_letters() {
        let policy = RetryPolicy::new(3);
        assert_eq!(policy.on_failure(true, 3), FailureAction::DeadLetter);
    }

    #[test]
    fn permanent_failure_dead_letters_immediately() {
        let policy = RetryPolicy::new(3);
        assert_eq!(policy.on_failure(false, 1), FailureAction::DeadLetter);
    }

    #[test]
    fn retry_delay_follows_backoff() {
        let policy = RetryPolicy::new(3).with_backoff(Backoff::new(100, 2, 10_000).unwrap());
        assert_eq!(policy.retry_delay_ms(1), 200);
        assert_eq!(policy.retry_delay_ms(2), 400);
    }
}
