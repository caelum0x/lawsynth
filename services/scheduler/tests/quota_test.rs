//! Integration tests for per-pool resource quota enforcement.
//!
//! Quota arithmetic is exercised here through its real consumer, the
//! [`PoolRegistry`], which admits and releases reservations against a pool's
//! capacity exactly as the scheduler does during leasing.

use lawsynth_runner::ResourceRequest;
use lawsynth_scheduler::{PoolRegistry, SchedulerError, WorkerPool};

fn registry() -> PoolRegistry {
    let mut registry = PoolRegistry::new();
    registry
        .register(WorkerPool::new("cpu-a", ResourceRequest::new(500, 4096, 4096).unwrap()).unwrap())
        .unwrap();
    registry
}

#[test]
fn available_capacity_tracks_reservations_exactly() {
    let mut registry = registry();
    registry.reserve("cpu-a", ResourceRequest::new(200, 1024, 512).unwrap()).unwrap();
    registry.reserve("cpu-a", ResourceRequest::new(100, 512, 512).unwrap()).unwrap();
    assert_eq!(
        registry.available("cpu-a").unwrap(),
        ResourceRequest::new(200, 2560, 3072).unwrap()
    );
}

#[test]
fn a_job_exceeding_the_quota_is_refused() {
    let mut registry = registry();
    // Fill most of the pool, then a further large request must be refused.
    registry.reserve("cpu-a", ResourceRequest::new(450, 4000, 4000).unwrap()).unwrap();
    let error =
        registry.reserve("cpu-a", ResourceRequest::new(100, 200, 200).unwrap()).unwrap_err();
    assert!(matches!(error, SchedulerError::InvalidWorker(_)));
    // The refused reservation left the running total unchanged.
    assert_eq!(registry.reserved("cpu-a"), Some(ResourceRequest::new(450, 4000, 4000).unwrap()));
}

#[test]
fn a_job_that_exactly_fills_the_quota_is_admitted() {
    let mut registry = registry();
    registry.reserve("cpu-a", ResourceRequest::new(500, 4096, 4096).unwrap()).unwrap();
    assert_eq!(
        registry.available("cpu-a").unwrap(),
        ResourceRequest { cpu_millis: 0, memory_bytes: 0, disk_bytes: 0 }
    );
}

#[test]
fn releasing_more_than_reserved_is_a_corrupt_invariant() {
    let mut registry = registry();
    registry.reserve("cpu-a", ResourceRequest::new(100, 512, 512).unwrap()).unwrap();
    let error =
        registry.release("cpu-a", ResourceRequest::new(200, 512, 512).unwrap()).unwrap_err();
    assert!(matches!(error, SchedulerError::CorruptCheckpoint(_)));
}

#[test]
fn fits_within_matches_the_admission_decision() {
    let available = registry().available("cpu-a").unwrap();
    assert!(ResourceRequest::new(500, 4096, 4096).unwrap().fits_within(available));
    assert!(!ResourceRequest::new(501, 4096, 4096).unwrap().fits_within(available));
}
