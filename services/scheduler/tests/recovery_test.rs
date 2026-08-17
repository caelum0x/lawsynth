//! Integration tests for expired-lease recovery.

mod common;

use lawsynth_scheduler::{JobState, RecoveryOutcome, on_lease_expiry};

use common::{scheduler_with_pool, simulation_job};

#[test]
fn decision_requeues_while_attempts_remain_and_deadline_is_live() {
    assert_eq!(on_lease_expiry(1, 3, false), RecoveryOutcome::Requeue);
}

#[test]
fn decision_dead_letters_when_attempts_exhausted() {
    let RecoveryOutcome::DeadLetter { reason } = on_lease_expiry(3, 3, false) else {
        panic!("expected dead letter");
    };
    assert!(reason.contains("without a heartbeat"));
}

#[test]
fn decision_dead_letters_when_deadline_elapsed() {
    let RecoveryOutcome::DeadLetter { reason } = on_lease_expiry(1, 3, true) else {
        panic!("expected dead letter");
    };
    assert!(reason.contains("while leased"));
}

#[test]
fn an_expired_lease_returns_the_job_to_the_queue() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("recov", 1_000), 10).unwrap();
    let lease = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    assert_eq!(lease.expires_at_ms, 70);

    // Recovering after the lease deadline requeues the job with a bumped attempt.
    assert_eq!(scheduler.recover_expired(71).unwrap(), 1);
    assert_eq!(scheduler.state("recov").unwrap(), &JobState::Queued);
    let retry = scheduler.lease_next("cpu-a", 72).unwrap().unwrap();
    assert_eq!(retry.envelope.work.attempt, 2);
}

#[test]
fn an_exhausted_lease_reaches_dead_letter() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("exhaust", 10_000), 10).unwrap();
    // First lease + expiry -> requeue (attempt 2 of max 2).
    scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    assert_eq!(scheduler.recover_expired(71).unwrap(), 1);
    // Second lease + expiry -> attempts spent -> dead letter.
    scheduler.lease_next("cpu-a", 72).unwrap().unwrap();
    assert_eq!(scheduler.recover_expired(123).unwrap(), 1);
    assert!(matches!(scheduler.state("exhaust").unwrap(), JobState::DeadLetter { .. }));
}

#[test]
fn a_queued_job_past_its_deadline_is_dead_lettered_on_recovery() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("stale-queue", 30), 10).unwrap();
    // No lease taken; recovering after the deadline expires the queued job.
    scheduler.recover_expired(31).unwrap();
    let JobState::DeadLetter { reason } = scheduler.state("stale-queue").unwrap() else {
        panic!("expected dead letter");
    };
    assert!(reason.contains("before assignment"));
}
