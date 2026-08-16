use lawsynth_runner::ResourceRequest;
use lawsynth_store::{LocalStore, StoreConfig};
use lawsynth_worker::{Worker, WorkerConfig, WorkerError};

#[test]
fn corrupt_persisted_checkpoint_is_detected_instead_of_being_silently_reused() {
    let root =
        std::env::temp_dir().join(format!("lawsynth-worker-checkpoint-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("worker/checkpoints")).unwrap();
    std::fs::write(root.join("worker/checkpoints/bad.checkpoint"), b"version=7\n").unwrap();
    let worker = Worker::new(
        WorkerConfig::new(ResourceRequest::new(1, 1, 0).unwrap(), 1024).unwrap(),
        LocalStore::open(&root, StoreConfig::default()).unwrap(),
    )
    .unwrap();
    assert!(matches!(worker.checkpoint("bad"), Err(WorkerError::CorruptCheckpoint(_))));
    let _ = std::fs::remove_dir_all(root);
}
