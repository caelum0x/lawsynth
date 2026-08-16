use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::DiscoveryConfig;
use lawsynth_runner::{CancellationToken, ResourceRequest};
use lawsynth_store::{LocalStore, StoreConfig};
use lawsynth_worker::{Job, JobEnvelope, JobOutput, Worker, WorkerConfig};

#[test]
fn executes_real_sparse_discovery_against_an_aligned_numeric_dataset() {
    let root =
        std::env::temp_dir().join(format!("lawsynth-worker-discovery-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let x = Identifier::new("x").unwrap();
    let time = (0..101).map(|index| index as f64 * 0.01).collect::<Vec<_>>();
    let values = time.iter().map(|time| (2.0 * time).exp()).collect::<Vec<_>>();
    let dataset =
        Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(x.clone(), values)])
            .unwrap();
    let envelope = JobEnvelope::new(
        "discovery-1",
        1,
        1,
        10_000,
        ResourceRequest::new(250, 1 << 20, 1 << 20).unwrap(),
        Job::Discover { dataset, config: DiscoveryConfig::new([x]) },
    )
    .unwrap();
    let worker = Worker::new(
        WorkerConfig::new(ResourceRequest::new(1_000, 2 << 20, 2 << 20).unwrap(), 1024).unwrap(),
        LocalStore::open(&root, StoreConfig::default()).unwrap(),
    )
    .unwrap();
    let output = worker.execute_at(&envelope, &CancellationToken::default(), 2).unwrap();
    let JobOutput::Discovery(result) = output else {
        panic!("worker dispatched discovery to the wrong executor")
    };
    assert!(!result.candidates.is_empty());
    let _ = std::fs::remove_dir_all(root);
}
