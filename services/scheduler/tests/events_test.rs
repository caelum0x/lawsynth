//! Integration tests for the lifecycle event log.

mod common;

use lawsynth_scheduler::JobEvent;

use common::{scheduler_with_pool, simulation_job};

#[test]
fn submit_lease_complete_emit_ordered_events() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("e1", 1_000), 10).unwrap();
    let lease = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    scheduler.complete(&lease.token, 30).unwrap();

    let records: Vec<_> = scheduler.events().records().collect();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].sequence, 1);
    assert_eq!(records[1].sequence, 2);
    assert_eq!(records[2].sequence, 3);
    assert!(records.iter().all(|record| record.job_id == "e1"));
    assert_eq!(records[0].event, JobEvent::Queued);
    assert_eq!(records[1].event, JobEvent::Leased { worker_id: "cpu-a".into(), generation: 1 });
    assert_eq!(records[2].event, JobEvent::Completed);
    assert_eq!(records[1].at_ms, 20);
}

#[test]
fn cancellation_emits_a_cancelled_event_with_reason() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("e2", 1_000), 10).unwrap();
    scheduler.cancel("e2", "operator stop", 15).unwrap();
    let last = scheduler.events().records().last().unwrap();
    assert_eq!(last.event, JobEvent::Cancelled { reason: "operator stop".into() });
}

#[test]
fn dead_letter_transitions_emit_a_dead_letter_event() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("e3", 10_000), 10).unwrap();
    // Exhaust the two attempts through retryable failures.
    let first = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    scheduler.fail(&first.token, true, "transient", 21).unwrap();
    let second = scheduler.lease_next("cpu-a", 22).unwrap().unwrap();
    scheduler.fail(&second.token, true, "final", 23).unwrap();

    let last = scheduler.events().records().last().unwrap();
    assert_eq!(last.event.name(), "dead_letter");
}

#[test]
fn since_replays_only_newer_events() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("e4", 1_000), 10).unwrap();
    let lease = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    scheduler.complete(&lease.token, 30).unwrap();

    let names: Vec<&str> = scheduler.events().since(1).map(|record| record.event.name()).collect();
    assert_eq!(names, vec!["leased", "completed"]);
}
