//! Worker-side lease and fencing tests. A later lease for the same job always
//! carries a higher generation, so a stale worker is fenced and must not commit.

use lawsynth_worker::{LeaseState, LeaseToken, WorkerError, WorkerLease};

fn token(job: &str, worker: &str, generation: u64) -> LeaseToken {
    LeaseToken::new(job, worker, generation).unwrap()
}

#[test]
fn rejects_non_url_safe_identifiers() {
    assert!(matches!(LeaseToken::new("bad id", "w1", 1), Err(WorkerError::InvalidJob(_))));
    assert!(matches!(LeaseToken::new("job", "worker/1", 1), Err(WorkerError::InvalidJob(_))));
}

#[test]
fn a_lease_is_held_until_its_expiry() {
    let lease = WorkerLease::new(token("job-a", "w1", 1), 100, 200).unwrap();
    assert!(!lease.is_expired(150));
    assert_eq!(lease.state(150), LeaseState::Held);
    assert!(lease.is_expired(200));
    assert_eq!(lease.state(200), LeaseState::Expired);
}

#[test]
fn expiry_must_follow_issuance() {
    assert!(matches!(
        WorkerLease::new(token("job-a", "w1", 1), 200, 200),
        Err(WorkerError::InvalidJob(_))
    ));
}

#[test]
fn a_newer_generation_fences_the_stale_worker() {
    let stale = WorkerLease::new(token("job-a", "w1", 1), 100, 200).unwrap();
    let authoritative = token("job-a", "w2", 2);

    // A newer generation for the same job supersedes the stale token.
    assert!(stale.token.is_superseded_by(&authoritative));
    // The stale worker may not commit against the newer authoritative token.
    assert!(!stale.may_commit(&authoritative));
    // The current holder at the same generation may commit.
    let current = WorkerLease::new(token("job-a", "w2", 2), 200, 300).unwrap();
    assert!(current.may_commit(&authoritative));
}

#[test]
fn renewal_extends_expiry_without_changing_the_fencing_token() {
    let lease = WorkerLease::new(token("job-a", "w1", 3), 100, 200).unwrap();
    let renewed = lease.renew(150, 100).unwrap();
    assert_eq!(renewed.token, lease.token);
    assert_eq!(renewed.issued_at_ms, 150);
    assert_eq!(renewed.expires_at_ms, 250);

    // A lease that has already lapsed cannot be renewed.
    assert!(matches!(lease.renew(200, 100), Err(WorkerError::LimitExceeded(_))));
}
