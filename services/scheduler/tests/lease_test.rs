//! Integration tests for lease acquisition, heartbeats, and fencing.

mod common;

use lawsynth_scheduler::{SchedulerError, SchedulerTransport};

use common::{scheduler_with_pool, simulation_job};

#[test]
fn a_fresh_lease_carries_a_first_generation_fencing_token() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("l1", 1_000), 10).unwrap();
    let lease = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    assert_eq!(lease.token.job_id, "l1");
    assert_eq!(lease.token.worker_id, "cpu-a");
    assert_eq!(lease.token.generation, 1);
    // Lease expiry is bounded by the lease duration, capped by the deadline.
    assert_eq!(lease.expires_at_ms, 70);
    assert_eq!(lease.issued_at_ms, 20);
}

#[test]
fn a_heartbeat_extends_a_live_lease() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("l2", 1_000), 10).unwrap();
    let lease = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    let extended = scheduler.heartbeat(&lease.token, 40).unwrap();
    assert_eq!(extended.expires_at_ms, 90);
}

#[test]
fn a_stale_token_is_fenced_after_recovery_reissues_the_lease() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("l3", 1_000), 10).unwrap();
    let first = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    // Let the lease expire and be recovered, then re-lease at a higher generation.
    assert_eq!(scheduler.recover_expired(71).unwrap(), 1);
    let second = scheduler.lease_next("cpu-a", 72).unwrap().unwrap();
    assert_eq!(second.token.generation, 2);

    // The original (fenced) token can neither complete nor heartbeat.
    assert!(matches!(scheduler.complete(&first.token, 73), Err(SchedulerError::StaleLease { .. })));
    assert!(matches!(
        scheduler.heartbeat(&first.token, 73),
        Err(SchedulerError::StaleLease { .. })
    ));
    scheduler.complete(&second.token, 74).unwrap();
}

#[test]
fn leasing_reserves_pool_capacity_so_a_third_job_waits() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("a", 1_000), 10).unwrap();
    scheduler.submit(simulation_job("b", 1_000), 10).unwrap();
    scheduler.submit(simulation_job("c", 1_000), 10).unwrap();
    // Pool holds 500 cpu; each job needs 250, so only two fit concurrently.
    assert!(scheduler.lease_next("cpu-a", 20).unwrap().is_some());
    assert!(scheduler.lease_next("cpu-a", 20).unwrap().is_some());
    assert!(scheduler.lease_next("cpu-a", 20).unwrap().is_none());
}

#[test]
fn transport_surface_reports_availability_honestly() {
    assert!(SchedulerTransport::LocalTyped.is_available());
    assert!(SchedulerTransport::HttpControlPlane.is_available());
    assert!(!SchedulerTransport::BrokerNotLinked.is_available());
    assert!(!SchedulerTransport::NetworkNotLinked.is_available());
    assert!(SchedulerTransport::BrokerNotLinked.reason().contains("no broker"));
}
