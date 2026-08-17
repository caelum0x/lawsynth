//! Integration tests for [`CheckpointStore`] durable persistence.

use lawsynth_scheduler::{CheckpointStore, JobState, PersistedCheckpoint, SchedulerError};
use lawsynth_store::{MemoryStore, ObjectKey, ObjectStore};

fn checkpoint(job_id: &str, state: JobState, sequence: u64) -> PersistedCheckpoint {
    PersistedCheckpoint {
        job_id: job_id.to_owned(),
        attempt: 1,
        sequence,
        updated_at_ms: 1_000,
        state,
    }
}

#[test]
fn writes_and_reads_back_every_lifecycle_state() {
    let store = CheckpointStore::new(MemoryStore::default(), 8192);
    for (index, state) in [
        JobState::Queued,
        JobState::Leased { worker_id: "cpu-a".into(), generation: 3, expires_at_ms: 500 },
        JobState::Completed,
        JobState::Cancelled { reason: "operator stop".into() },
        JobState::DeadLetter { reason: "attempts exhausted".into() },
    ]
    .into_iter()
    .enumerate()
    {
        let job_id = format!("job-{index}");
        let original = checkpoint(&job_id, state, index as u64 + 1);
        store.write(&original).unwrap();
        assert_eq!(store.read(&job_id).unwrap().unwrap(), original);
    }
}

#[test]
fn reading_an_absent_job_yields_none() {
    let store = CheckpointStore::new(MemoryStore::default(), 8192);
    assert_eq!(store.read("nobody").unwrap(), None);
}

#[test]
fn a_later_write_overwrites_the_previous_checkpoint() {
    let store = CheckpointStore::new(MemoryStore::default(), 8192);
    store.write(&checkpoint("job-1", JobState::Queued, 1)).unwrap();
    store.write(&checkpoint("job-1", JobState::Completed, 2)).unwrap();
    let read = store.read("job-1").unwrap().unwrap();
    assert_eq!(read.state, JobState::Completed);
    assert_eq!(read.sequence, 2);
}

#[test]
fn rejects_a_record_larger_than_the_ceiling() {
    let store = CheckpointStore::new(MemoryStore::default(), 16);
    let error = store.write(&checkpoint("job-1", JobState::Queued, 1)).unwrap_err();
    assert!(matches!(error, SchedulerError::CheckpointTooLarge { .. }));
}

#[test]
fn corrupt_bytes_are_rejected_on_read() {
    let checkpoints = CheckpointStore::new(MemoryStore::default(), 8192);
    checkpoints.write(&checkpoint("job-1", JobState::Queued, 1)).unwrap();
    // Overwrite the durable object with garbage through the borrowed store handle.
    let key = ObjectKey::new("scheduler/checkpoints/job-1.state").unwrap();
    checkpoints.store().put(key, b"not-a-checkpoint".to_vec()).unwrap();
    assert!(matches!(checkpoints.read("job-1"), Err(SchedulerError::CorruptCheckpoint(_))));
}
