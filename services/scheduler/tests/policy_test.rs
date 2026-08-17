//! Integration tests for the retry / dead-letter [`RetryPolicy`] and its backoff.

use lawsynth_scheduler::{Backoff, FailureAction, RetryPolicy};

#[test]
fn retryable_failures_requeue_until_the_budget_is_spent() {
    let policy = RetryPolicy::new(3);
    assert_eq!(policy.on_failure(true, 1), FailureAction::Requeue);
    assert_eq!(policy.on_failure(true, 2), FailureAction::Requeue);
    assert_eq!(policy.on_failure(true, 3), FailureAction::DeadLetter);
}

#[test]
fn permanent_failures_dead_letter_on_the_first_attempt() {
    let policy = RetryPolicy::new(5);
    assert_eq!(policy.on_failure(false, 1), FailureAction::DeadLetter);
}

#[test]
fn maximum_attempts_is_reported() {
    assert_eq!(RetryPolicy::new(4).maximum_attempts(), 4);
}

#[test]
fn retry_delay_follows_the_configured_backoff() {
    let policy = RetryPolicy::new(3).with_backoff(Backoff::new(100, 2, 10_000).unwrap());
    // Delay before the attempt that follows attempt N.
    assert_eq!(policy.retry_delay_ms(1), 200);
    assert_eq!(policy.retry_delay_ms(2), 400);
    assert_eq!(policy.retry_delay_ms(3), 800);
}

#[test]
fn backoff_caps_and_is_monotonic() {
    let backoff = Backoff::new(100, 3, 1_000).unwrap();
    let schedule = backoff.schedule(5);
    assert_eq!(schedule, vec![100, 300, 900, 1_000, 1_000]);
    assert!(schedule.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
fn backoff_rejects_degenerate_parameters() {
    assert!(Backoff::new(0, 2, 10).is_err());
    assert!(Backoff::new(10, 0, 10).is_err());
    assert!(Backoff::new(10, 2, 5).is_err());
}
