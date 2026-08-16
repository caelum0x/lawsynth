use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::{CancellationToken, ResourceRequest};
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_store::{LocalStore, StoreConfig};
use lawsynth_worker::{CheckpointState, Job, JobEnvelope, Worker, WorkerConfig, WorkerError};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

#[test]
fn capacity_is_rejected_before_a_job_can_enter_the_running_state() {
    let root = std::env::temp_dir().join(format!("lawsynth-worker-limits-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let store = LocalStore::open(&root, StoreConfig::default()).unwrap();
    let worker = Worker::new(
        WorkerConfig::new(ResourceRequest::new(1, 1024, 1024).unwrap(), 1024).unwrap(),
        store,
    )
    .unwrap();
    let x = Identifier::new("x").unwrap();
    let world = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(x.clone(), Expr::constant(1.0))],
    )
    .unwrap();
    let envelope = JobEnvelope::new(
        "over-capacity",
        1,
        1,
        10,
        ResourceRequest::new(2, 1, 1).unwrap(),
        Job::Simulate {
            world,
            config: SimulationConfig::new(0.0, 1.0, 0.1).unwrap(),
            request: SimulationRequest::default().with_initial(x, 0.0),
        },
    )
    .unwrap();
    assert!(matches!(
        worker.execute_at(&envelope, &CancellationToken::default(), 2),
        Err(WorkerError::Runner(_))
    ));
    assert_eq!(
        worker.checkpoint("over-capacity").unwrap().unwrap().state,
        CheckpointState::Rejected
    );
    let _ = std::fs::remove_dir_all(root);
}
