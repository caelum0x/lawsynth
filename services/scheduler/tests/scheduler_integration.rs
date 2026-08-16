use std::collections::BTreeMap;
use std::time::Duration;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::{CancellationToken, ResourceRequest};
use lawsynth_scheduler::{
    JobState, Scheduler, SchedulerConfig, SchedulerError, SchedulerTransport, WorkerPool,
};
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_store::{LocalStore, MemoryStore, StoreConfig};
use lawsynth_worker::{Job, JobEnvelope, JobOutput, Worker, WorkerConfig};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn resources() -> ResourceRequest {
    ResourceRequest::new(250, 1024, 1024).unwrap()
}

fn simulation_job(name: &str, deadline_at_ms: u64) -> JobEnvelope {
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
        deadline_at_ms,
        resources(),
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

fn scheduler() -> Scheduler<MemoryStore> {
    let config = SchedulerConfig::new(8, 2, Duration::from_millis(50), 8192).unwrap();
    let mut scheduler = Scheduler::new(config, MemoryStore::default()).unwrap();
    scheduler
        .register_pool(
            WorkerPool::new("cpu-a", ResourceRequest::new(500, 4096, 4096).unwrap()).unwrap(),
        )
        .unwrap();
    scheduler
}

#[test]
fn dispatches_worker_compatible_envelope_through_real_rk4_execution() {
    let mut scheduler = scheduler();
    scheduler.submit(simulation_job("real-rk4", 1_000), 10).unwrap();
    let lease = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    let root =
        std::env::temp_dir().join(format!("lawsynth-scheduler-worker-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let worker = Worker::new(
        WorkerConfig::new(ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap(), 1024).unwrap(),
        LocalStore::open(&root, StoreConfig::default()).unwrap(),
    )
    .unwrap();
    let result = worker.execute_at(&lease.envelope, &CancellationToken::default(), 21).unwrap();
    let JobOutput::Simulation(trajectory) = result else {
        panic!("scheduler issued the wrong work");
    };
    assert_eq!(trajectory.samples(), 101);
    scheduler.complete(&lease.token, 22).unwrap();
    assert_eq!(scheduler.state("real-rk4").unwrap(), &JobState::Completed);
    let checkpoint = scheduler.checkpoint("real-rk4").unwrap().unwrap();
    assert_eq!(checkpoint.state, JobState::Completed);
    assert_eq!(checkpoint.sequence, 3);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn lease_heartbeats_fence_stale_workers_and_recover_missing_workers() {
    let mut scheduler = scheduler();
    scheduler.submit(simulation_job("leased", 1_000), 10).unwrap();
    let first = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    let heartbeated = scheduler.heartbeat(&first.token, 40).unwrap();
    assert_eq!(heartbeated.expires_at_ms, 90);
    assert_eq!(scheduler.recover_expired(91).unwrap(), 1);
    let retry = scheduler.lease_next("cpu-a", 92).unwrap().unwrap();
    assert_eq!(retry.envelope.work.attempt, 2);
    assert!(matches!(scheduler.complete(&first.token, 93), Err(SchedulerError::StaleLease { .. })));
    scheduler.complete(&retry.token, 93).unwrap();
}

#[test]
fn resources_deadline_cancellation_and_deadletter_transitions_are_explicit() {
    let mut scheduler = scheduler();
    scheduler.submit(simulation_job("one", 1_000), 10).unwrap();
    scheduler.submit(simulation_job("two", 1_000), 10).unwrap();
    let first = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    let second = scheduler.lease_next("cpu-a", 20).unwrap().unwrap();
    assert!(scheduler.lease_next("cpu-a", 20).unwrap().is_none());
    scheduler.cancel("two", "operator request", 21).unwrap();
    assert_eq!(
        scheduler.state("two").unwrap(),
        &JobState::Cancelled { reason: "operator request".into() }
    );
    assert_eq!(
        scheduler.fail(&first.token, true, "transient worker fault", 22).unwrap(),
        JobState::Queued
    );
    let retry = scheduler.lease_next("cpu-a", 23).unwrap().unwrap();
    assert_eq!(retry.envelope.work.attempt, 2);
    assert!(matches!(
        scheduler.fail(&retry.token, true, "last transient fault", 24).unwrap(),
        JobState::DeadLetter { .. }
    ));
    scheduler.submit(simulation_job("expired", 30), 10).unwrap();
    assert!(scheduler.lease_next("cpu-a", 30).unwrap().is_none());
    assert!(matches!(scheduler.state("expired").unwrap(), JobState::DeadLetter { .. }));
    assert!(!SchedulerTransport::BrokerNotLinked.is_available());
    assert!(SchedulerTransport::BrokerNotLinked.reason().contains("no broker"));
    drop(second);
}

#[test]
fn checkpoints_survive_reopen_and_reject_corruption() {
    let root =
        std::env::temp_dir().join(format!("lawsynth-scheduler-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let config = SchedulerConfig::new(4, 2, Duration::from_secs(1), 8192).unwrap();
    let store = LocalStore::open(&root, StoreConfig::default()).unwrap();
    let mut scheduler = Scheduler::new(config.clone(), store).unwrap();
    scheduler
        .register_pool(
            WorkerPool::new("cpu-a", ResourceRequest::new(500, 4096, 4096).unwrap()).unwrap(),
        )
        .unwrap();
    scheduler.submit(simulation_job("durable", 1_000), 10).unwrap();
    scheduler.cancel("durable", "audit stop", 11).unwrap();
    drop(scheduler);
    let reopened =
        Scheduler::new(config, LocalStore::open(&root, StoreConfig::default()).unwrap()).unwrap();
    assert_eq!(
        reopened.checkpoint("durable").unwrap().unwrap().state,
        JobState::Cancelled { reason: "audit stop".into() }
    );
    std::fs::write(root.join("scheduler/checkpoints/durable.state"), b"not-a-checkpoint").unwrap();
    assert!(matches!(reopened.checkpoint("durable"), Err(SchedulerError::CorruptCheckpoint(_))));
    let _ = std::fs::remove_dir_all(root);
}
