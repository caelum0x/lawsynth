//! Integration tests for [`WorkerPool`] validation and the [`PoolRegistry`].

use lawsynth_runner::ResourceRequest;
use lawsynth_scheduler::{PoolRegistry, SchedulerError, WorkerPool};

fn capacity() -> ResourceRequest {
    ResourceRequest::new(500, 4096, 4096).unwrap()
}

fn request() -> ResourceRequest {
    ResourceRequest::new(250, 1024, 1024).unwrap()
}

#[test]
fn worker_pool_requires_a_url_safe_id() {
    assert!(WorkerPool::new("cpu-a_1", capacity()).is_ok());
    assert!(WorkerPool::new("cpu a", capacity()).is_err());
    assert!(WorkerPool::new("", capacity()).is_err());
    assert!(WorkerPool::new("x".repeat(129), capacity()).is_err());
}

#[test]
fn registry_registers_pools_uniquely() {
    let mut registry = PoolRegistry::new();
    assert!(registry.is_empty());
    registry.register(WorkerPool::new("cpu-a", capacity()).unwrap()).unwrap();
    assert!(registry.contains("cpu-a"));
    assert_eq!(registry.len(), 1);

    let duplicate = registry.register(WorkerPool::new("cpu-a", capacity()).unwrap()).unwrap_err();
    assert!(matches!(duplicate, SchedulerError::InvalidWorker(_)));
}

#[test]
fn reservations_reduce_and_releases_restore_availability() {
    let mut registry = PoolRegistry::new();
    registry.register(WorkerPool::new("cpu-a", capacity()).unwrap()).unwrap();
    assert_eq!(registry.available("cpu-a").unwrap(), capacity());

    registry.reserve("cpu-a", request()).unwrap();
    assert_eq!(registry.reserved("cpu-a"), Some(request()));
    assert_eq!(
        registry.available("cpu-a").unwrap(),
        ResourceRequest::new(250, 3072, 3072).unwrap()
    );

    registry.release("cpu-a", request()).unwrap();
    assert_eq!(registry.available("cpu-a").unwrap(), capacity());
}

#[test]
fn over_reservation_is_refused_as_an_invalid_worker() {
    let mut registry = PoolRegistry::new();
    registry.register(WorkerPool::new("cpu-a", capacity()).unwrap()).unwrap();
    registry.reserve("cpu-a", ResourceRequest::new(400, 4000, 4000).unwrap()).unwrap();
    let error = registry.reserve("cpu-a", request()).unwrap_err();
    assert!(matches!(error, SchedulerError::InvalidWorker(_)));
}

#[test]
fn operations_on_an_unknown_pool_are_reported() {
    let mut registry = PoolRegistry::new();
    assert!(matches!(registry.available("ghost"), Err(SchedulerError::UnknownWorker(_))));
    assert!(matches!(registry.reserve("ghost", request()), Err(SchedulerError::UnknownWorker(_))));
}

#[test]
fn ids_are_reported_in_stable_order() {
    let mut registry = PoolRegistry::new();
    registry.register(WorkerPool::new("cpu-c", capacity()).unwrap()).unwrap();
    registry.register(WorkerPool::new("cpu-a", capacity()).unwrap()).unwrap();
    registry.register(WorkerPool::new("cpu-b", capacity()).unwrap()).unwrap();
    let ids: Vec<&str> = registry.ids().collect();
    assert_eq!(ids, vec!["cpu-a", "cpu-b", "cpu-c"]);
}
