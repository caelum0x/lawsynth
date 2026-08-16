use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::{CancellationToken, ResourceRequest};
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_store::{LocalStore, StoreConfig};
use lawsynth_worker::{
    CheckpointState, Job, JobEnvelope, JobOutput, TransportSurface, Worker, WorkerConfig,
    WorkerError,
};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn root(label: &str) -> std::path::PathBuf {
    let unique = format!("lawsynth-worker-{label}-{}", std::process::id());
    let path = std::env::temp_dir().join(unique);
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn worker(label: &str) -> (Worker<LocalStore>, std::path::PathBuf) {
    let path = root(label);
    let store = LocalStore::open(&path, StoreConfig::default()).unwrap();
    let config =
        WorkerConfig::new(ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap(), 1 << 10).unwrap();
    (Worker::new(config, store).unwrap(), path)
}

fn simulation_job(name: &str) -> JobEnvelope {
    let x = id("x");
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
fn executes_a_real_rk4_job_and_persists_completed_lifecycle() {
    let (worker, path) = worker("simulation");
    let envelope = simulation_job("simulation-1");
    let output = worker.execute_at(&envelope, &CancellationToken::default(), 20).unwrap();
    let JobOutput::Simulation(trajectory) = output else { panic!("expected simulation output") };
    assert_eq!(trajectory.samples(), 101);
    assert!((trajectory.values[&id("x")].last().unwrap() - std::f64::consts::E).abs() < 1e-8);

    let checkpoint = worker.checkpoint("simulation-1").unwrap().unwrap();
    assert_eq!(checkpoint.sequence, 2);
    assert_eq!(checkpoint.state, CheckpointState::Completed);
    drop(worker);
    let reopened = Worker::new(
        WorkerConfig::new(ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap(), 1 << 10).unwrap(),
        LocalStore::open(&path, StoreConfig::default()).unwrap(),
    )
    .unwrap();
    assert_eq!(reopened.checkpoint("simulation-1").unwrap().unwrap(), checkpoint);
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn rejects_cancelled_deadline_and_unsupported_transport_without_running_work() {
    let (worker, path) = worker("rejections");
    let cancelled = CancellationToken::default();
    cancelled.cancel("operator stopped job").unwrap();
    assert!(matches!(
        worker.execute_at(&simulation_job("cancelled-1"), &cancelled, 20),
        Err(WorkerError::Cancelled(_))
    ));
    assert_eq!(
        worker.checkpoint("cancelled-1").unwrap().unwrap().state,
        CheckpointState::Cancelled
    );
    assert!(matches!(
        worker.execute_at(&simulation_job("expired-1"), &CancellationToken::default(), 1_000),
        Err(WorkerError::DeadlineExceeded { .. })
    ));
    assert_eq!(worker.checkpoint("expired-1").unwrap().unwrap().state, CheckpointState::Rejected);
    assert!(!TransportSurface::QueueNotImplemented.is_available());
    assert!(TransportSurface::QueueNotImplemented.reason().contains("no queue"));
    assert!(!TransportSurface::NetworkNotImplemented.is_available());
    let _ = std::fs::remove_dir_all(path);
}
