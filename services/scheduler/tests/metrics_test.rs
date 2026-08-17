//! Integration tests proving the lifecycle counters track real transitions.

mod common;

use common::{scheduler_with_pool, simulation_job};

#[test]
fn counters_are_zero_before_any_work() {
    let scheduler = scheduler_with_pool();
    let snapshot = scheduler.metrics();
    assert_eq!(snapshot.queued, 0);
    assert_eq!(snapshot.leased, 0);
    assert_eq!(snapshot.completed, 0);
}

#[test]
fn a_full_lifecycle_updates_every_counter() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("aaa", 10_000), 10).unwrap();
    scheduler.submit(simulation_job("bbb", 10_000), 10).unwrap();
    assert_eq!(scheduler.metrics().queued, 2);

    // Complete the first job.
    let first = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    assert_eq!(scheduler.metrics().leased, 1);
    scheduler.complete(&first.token, 30).unwrap();
    assert_eq!(scheduler.metrics().completed, 1);

    // Fail the second retryably (requeue), then again to exhaust it (dead letter).
    let second = scheduler.lease_next("cpu-a", 31).unwrap().unwrap();
    scheduler.fail(&second.token, true, "transient", 32).unwrap();
    let retry = scheduler.lease_next("cpu-a", 33).unwrap().unwrap();
    scheduler.fail(&retry.token, true, "final", 34).unwrap();

    let snapshot = scheduler.metrics();
    assert_eq!(snapshot.queued, 3); // two submissions plus one requeue
    assert_eq!(snapshot.leased, 3); // first, second, retry
    assert_eq!(snapshot.completed, 1);
    assert_eq!(snapshot.failed, 2);
    assert_eq!(snapshot.dead_letter, 1);
    assert_eq!(snapshot.cancelled, 0);
}

#[test]
fn cancellation_increments_the_cancelled_counter() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("ccc", 10_000), 10).unwrap();
    scheduler.cancel("ccc", "operator stop", 15).unwrap();
    assert_eq!(scheduler.metrics().cancelled, 1);
}
