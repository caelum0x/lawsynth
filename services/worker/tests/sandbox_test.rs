//! Sandbox guard tests. The guard enforces the portable, deterministic bounds a
//! worker owns -- the wall-clock deadline and the configured per-job resource
//! ceilings -- with an injected clock. It does not claim OS-level isolation.

use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::ResourceRequest;
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_worker::{Job, JobEnvelope, Limits, Sandbox, WorkerConfig, WorkerError};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

/// A runnable simulation envelope mirroring the worker's own execution tests.
/// Submitted at 10 ms with a 1000 ms deadline and a 250 cpu-milli request.
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

fn base_limits() -> Limits {
    let capacity = ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap();
    Limits::from_config(&WorkerConfig::new(capacity, 1 << 10).unwrap())
}

#[test]
fn baseline_sandbox_admits_a_live_job_and_rejects_an_expired_one() {
    let sandbox = Sandbox::new(base_limits());
    let envelope = simulation_job("sim-live");
    // Before the deadline: admitted.
    assert!(sandbox.admit(&envelope.work, 20).is_ok());
    // At/after the deadline: rejected with the same error the worker reports.
    assert!(matches!(
        sandbox.admit(&envelope.work, 1_000),
        Err(WorkerError::DeadlineExceeded { .. })
    ));
}

#[test]
fn a_configured_resource_ceiling_rejects_an_oversized_job() {
    let ceiling = ResourceRequest::new(100, 1 << 19, 1 << 19).unwrap();
    let limits = base_limits().with_max_job_resources(ceiling).unwrap();
    let sandbox = Sandbox::new(limits);
    let envelope = simulation_job("sim-too-big");
    // The job requests 250 cpu-millis against a 100 ceiling.
    assert!(matches!(sandbox.admit(&envelope.work, 20), Err(WorkerError::LimitExceeded(_))));
}

#[test]
fn a_configured_wall_ceiling_rejects_a_long_deadline() {
    // Job wall budget is 1000 - 10 = 990 ms; cap it at 500 ms.
    let limits = base_limits().with_max_wall_ms(500).unwrap();
    let sandbox = Sandbox::new(limits);
    let envelope = simulation_job("sim-long");
    assert!(matches!(sandbox.admit(&envelope.work, 20), Err(WorkerError::LimitExceeded(_))));
}

#[test]
fn overrun_detection_reports_a_deadline_blown_during_execution() {
    let sandbox = Sandbox::new(base_limits());
    let envelope = simulation_job("sim-overrun");
    // Admitted at 20 ms, but the clock read after work finished is past the
    // 1000 ms deadline.
    assert!(sandbox.check_deadline(&envelope.work, 20).is_ok());
    assert!(matches!(
        sandbox.check_overrun(&envelope.work, 1_500),
        Err(WorkerError::DeadlineExceeded { .. })
    ));
}
