//! Artifact upload tests. A completed job's output manifest is uploaded through
//! the checksum-verified path and must survive a store round-trip with a
//! matching digest.

use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::{CancellationToken, ResourceRequest};
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_store::{LocalStore, ObjectKey, ObjectStore, StoreConfig};
use lawsynth_worker::{Job, JobEnvelope, Worker, WorkerConfig};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

fn root(label: &str) -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("lawsynth-worker-upload-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn simulation_job(name: &str) -> JobEnvelope {
    let x = Identifier::new("x").unwrap();
    let world = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(x.clone(), Expr::symbol(x.clone()))],
    )
    .unwrap();
    JobEnvelope::new(
        name,
        1,
        10,
        1_000,
        ResourceRequest::new(250, 1024, 1024).unwrap(),
        Job::Simulate {
            world,
            config: SimulationConfig::new(0.0, 1.0, 0.01).unwrap(),
            request: SimulationRequest {
                initial_state: BTreeMap::from([(x, 1.0)]),
                ..Default::default()
            },
        },
    )
    .unwrap()
}

#[test]
fn records_a_completed_jobs_artifact_and_verifies_it_round_trips() {
    let path = root("record");
    let store = LocalStore::open(&path, StoreConfig::default()).unwrap();
    let config =
        WorkerConfig::new(ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap(), 1 << 12).unwrap();
    let worker = Worker::new(config, store).unwrap();

    let output = worker
        .execute_at(&simulation_job("sim-upload"), &CancellationToken::default(), 20)
        .unwrap();

    let receipt = worker.record_artifact("sim-upload", &output).unwrap();
    assert_eq!(receipt.job_id, "sim-upload");
    assert_eq!(receipt.kind, "simulate");
    assert_eq!(receipt.items, 101);
    assert!(receipt.upload.bytes > 0);
    assert_eq!(receipt.upload.key, "worker/artifacts/sim-upload/manifest");
    assert_ne!(receipt.upload.checksum, 0);

    // A successful handoff is counted in telemetry.
    assert_eq!(worker.telemetry().artifacts_uploaded, 1);

    // The manifest is durable and its stored checksum matches the receipt.
    drop(worker);
    let reopened = LocalStore::open(&path, StoreConfig::default()).unwrap();
    let stored = reopened.get(&ObjectKey::new(receipt.upload.key.clone()).unwrap()).unwrap();
    assert!(stored.verify());
    assert_eq!(stored.checksum, receipt.upload.checksum);
    assert_eq!(stored.len(), receipt.upload.bytes);

    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn a_discovery_artifact_records_its_candidate_count() {
    let path = root("discovery");
    let store = LocalStore::open(&path, StoreConfig::default()).unwrap();
    let config =
        WorkerConfig::new(ResourceRequest::new(1_000, 2 << 20, 2 << 20).unwrap(), 1 << 12).unwrap();
    let worker = Worker::new(config, store).unwrap();

    let x = Identifier::new("x").unwrap();
    let time = (0..101).map(|index| index as f64 * 0.01).collect::<Vec<_>>();
    let values = time.iter().map(|t| (2.0 * t).exp()).collect::<Vec<_>>();
    let dataset = lawsynth_data::Dataset::new(
        lawsynth_data::TimeAxis::new(time).unwrap(),
        [lawsynth_data::NumericColumn::new(x.clone(), values)],
    )
    .unwrap();
    let envelope = JobEnvelope::new(
        "disc-upload",
        1,
        1,
        10_000,
        ResourceRequest::new(250, 1 << 20, 1 << 20).unwrap(),
        Job::Discover { dataset, config: lawsynth_discovery::DiscoveryConfig::new([x]) },
    )
    .unwrap();

    let output = worker.execute_at(&envelope, &CancellationToken::default(), 2).unwrap();
    let receipt = worker.record_artifact("disc-upload", &output).unwrap();
    assert_eq!(receipt.kind, "discover");
    assert!(receipt.items >= 1);

    let _ = std::fs::remove_dir_all(path);
}
