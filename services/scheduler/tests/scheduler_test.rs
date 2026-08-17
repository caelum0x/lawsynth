//! End-to-end integration tests for the scheduler lifecycle.

mod common;

use std::time::Duration;

use lawsynth_scheduler::{JobState, Scheduler, SchedulerConfig, SchedulerError};
use lawsynth_store::MemoryStore;

use common::{scheduler_with_pool, simulation_job};

#[test]
fn submit_lease_complete_walks_a_job_to_completion() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("job-1", 1_000), 10).unwrap();
    assert_eq!(scheduler.state("job-1").unwrap(), &JobState::Queued);
    assert_eq!(scheduler.queued_count(), 1);

    let lease = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    assert!(matches!(scheduler.state("job-1").unwrap(), JobState::Leased { .. }));
    assert_eq!(scheduler.queued_count(), 0);

    scheduler.complete(&lease.token, 30).unwrap();
    assert_eq!(scheduler.state("job-1").unwrap(), &JobState::Completed);

    // The durable checkpoint reflects the terminal state.
    let checkpoint = scheduler.checkpoint("job-1").unwrap().unwrap();
    assert_eq!(checkpoint.state, JobState::Completed);
}

#[test]
fn duplicate_submissions_are_rejected() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("dup", 1_000), 10).unwrap();
    let error = scheduler.submit(simulation_job("dup", 1_000), 10).unwrap_err();
    assert!(matches!(error, SchedulerError::DuplicateJob(_)));
}

#[test]
fn a_full_queue_is_refused() {
    let config = SchedulerConfig::new(1, 2, Duration::from_millis(50), 8192).unwrap();
    let mut scheduler = Scheduler::new(config, MemoryStore::default()).unwrap();
    scheduler.submit(simulation_job("first", 1_000), 10).unwrap();
    let error = scheduler.submit(simulation_job("second", 1_000), 10).unwrap_err();
    assert!(matches!(error, SchedulerError::QueueFull { limit: 1 }));
}

#[test]
fn cancelling_a_queued_job_is_terminal() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("cancel-me", 1_000), 10).unwrap();
    scheduler.cancel("cancel-me", "operator stop", 15).unwrap();
    assert_eq!(
        scheduler.state("cancel-me").unwrap(),
        &JobState::Cancelled { reason: "operator stop".into() }
    );
    // Cancelling a terminal job is an invalid transition.
    let error = scheduler.cancel("cancel-me", "again", 16).unwrap_err();
    assert!(matches!(error, SchedulerError::InvalidTransition { .. }));
}

#[test]
fn an_unknown_job_has_no_state() {
    let scheduler = scheduler_with_pool();
    assert!(matches!(scheduler.state("ghost"), Err(SchedulerError::UnknownJob(_))));
}

#[test]
fn health_snapshot_reflects_live_queue_depth_and_metrics() {
    let mut scheduler = scheduler_with_pool();
    scheduler.submit(simulation_job("h1", 1_000), 10).unwrap();
    let snapshot = scheduler.health_snapshot();
    assert!(snapshot.ready);
    assert_eq!(snapshot.queued_count, 1);
    assert_eq!(snapshot.maximum_queued_jobs, 8);
    assert_eq!(snapshot.lease_duration_ms, 50);
    assert_eq!(snapshot.metrics.queued, 1);
}
