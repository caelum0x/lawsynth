use lawsynth_runner::ResourceRequest;
use lawsynth_worker::{WorkerConfig, WorkerError};

#[test]
fn worker_configuration_requires_a_checkpoint_large_enough_for_a_lifecycle_record() {
    let capacity = ResourceRequest::new(1, 1, 0).unwrap();
    assert!(matches!(WorkerConfig::new(capacity, 127), Err(WorkerError::InvalidConfig(_))));
    assert_eq!(WorkerConfig::new(capacity, 128).unwrap().capacity, capacity);
}
