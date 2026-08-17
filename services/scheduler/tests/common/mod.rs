//! Shared construction helpers for the scheduler integration tests.
//!
//! These mirror the executable [`JobEnvelope`] construction used by
//! `scheduler_integration.rs` so every integration test exercises the same real
//! typed work the scheduler dispatches. Not every test binary uses every helper,
//! so unused-item warnings are allowed here.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::time::Duration;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_runner::ResourceRequest;
use lawsynth_scheduler::{Scheduler, SchedulerConfig, WorkerPool};
use lawsynth_sim::{SimulationConfig, SimulationRequest};
use lawsynth_store::MemoryStore;
use lawsynth_worker::{Job, JobEnvelope};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

/// A validated identifier for building worlds.
pub fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

/// The standard per-job resource request used across the suite.
pub fn resources() -> ResourceRequest {
    ResourceRequest::new(250, 1024, 1024).unwrap()
}

/// Builds a real, worker-executable simulation envelope.
pub fn simulation_job(name: &str, deadline_at_ms: u64) -> JobEnvelope {
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

/// A memory-backed scheduler with one registered `cpu-a` pool, matching the
/// configuration used by the original integration test.
pub fn scheduler_with_pool() -> Scheduler<MemoryStore> {
    let config = SchedulerConfig::new(8, 2, Duration::from_millis(50), 8192).unwrap();
    let mut scheduler = Scheduler::new(config, MemoryStore::default()).unwrap();
    scheduler
        .register_pool(
            WorkerPool::new("cpu-a", ResourceRequest::new(500, 4096, 4096).unwrap()).unwrap(),
        )
        .unwrap();
    scheduler
}
