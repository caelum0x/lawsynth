//! Deterministic lease-heartbeat state machine tests. Time is injected in
//! milliseconds, so every "is a beat due?" and "has the lease lapsed?" decision
//! is reproducible with no wall-clock dependence.

use lawsynth_worker::{Heartbeat, HeartbeatState, WorkerError};

#[test]
fn rejects_an_interval_of_zero_or_a_ttl_below_one_interval() {
    assert!(matches!(Heartbeat::start(0, 0, 100), Err(WorkerError::InvalidConfig(_))));
    assert!(matches!(Heartbeat::start(0, 100, 50), Err(WorkerError::InvalidConfig(_))));
    assert!(Heartbeat::start(0, 100, 100).is_ok());
}

#[test]
fn is_fresh_before_the_interval_and_due_after_it() {
    let heartbeat = Heartbeat::start(1_000, 100, 500).unwrap();
    assert_eq!(heartbeat.sequence(), 1);
    assert_eq!(heartbeat.state(1_050), HeartbeatState::Fresh);
    assert!(!heartbeat.is_due(1_050));
    assert_eq!(heartbeat.state(1_100), HeartbeatState::Due);
    assert!(heartbeat.is_due(1_100));
}

#[test]
fn beating_advances_the_sequence_and_refreshes_the_lease() {
    let mut heartbeat = Heartbeat::start(1_000, 100, 500).unwrap();
    assert_eq!(heartbeat.expires_at_ms(), 1_500);

    let sequence = heartbeat.beat(1_120).unwrap();
    assert_eq!(sequence, 2);
    assert_eq!(heartbeat.last_beat_ms(), 1_120);
    assert_eq!(heartbeat.expires_at_ms(), 1_620);
    // Freshly beaten, so it is no longer due at the moment of the beat.
    assert_eq!(heartbeat.state(1_120), HeartbeatState::Fresh);
}

#[test]
fn a_lapsed_lease_reports_expired_and_refuses_to_beat() {
    let mut heartbeat = Heartbeat::start(1_000, 100, 500).unwrap();
    // TTL is 500 ms; well past it the lease has lapsed.
    assert!(heartbeat.is_expired(1_600));
    assert_eq!(heartbeat.state(1_600), HeartbeatState::Expired);
    assert!(matches!(heartbeat.beat(1_600), Err(WorkerError::LimitExceeded(_))));
    // The refused beat did not advance the sequence.
    assert_eq!(heartbeat.sequence(), 1);
}
