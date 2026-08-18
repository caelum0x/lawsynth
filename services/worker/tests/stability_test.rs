//! End-to-end worker dispatch of a stability-analysis job.
//!
//! Mirrors `execute_test.rs`, but drives the real `lawsynth-stability` engine
//! through the public [`Worker::execute_at`] path: a stable-node world is
//! analysed over a search box and the worker must return a
//! [`JobOutput::Stability`] with the located fixed point classified as a stable
//! node, proving the new variant flows through admission, execution, and the
//! artifact handoff.

use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator};
use lawsynth_runner::{CancellationToken, ResourceRequest};
use lawsynth_stability::{Classification, StabilityConfig};
use lawsynth_store::{LocalStore, StoreConfig};
use lawsynth_worker::{Job, JobEnvelope, JobOutput, Worker, WorkerConfig};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

/// A linear stable node at the origin: `x' = -x`, `y' = -2y`.
fn stable_node_world() -> World {
    World::new(
        [Variable::new(id("x"), VariableRole::State), Variable::new(id("y"), VariableRole::State)],
        [],
        [
            ContinuousLaw::new(id("x"), Expr::unary(UnaryOperator::Negate, Expr::symbol(id("x")))),
            ContinuousLaw::new(id("y"), Expr::product(Expr::constant(-2.0), Expr::symbol(id("y")))),
        ],
    )
    .unwrap()
}

#[test]
fn executes_real_stability_analysis_and_classifies_the_origin() {
    let root =
        std::env::temp_dir().join(format!("lawsynth-worker-stability-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let envelope = JobEnvelope::new(
        "stability-1",
        1,
        1,
        10_000,
        ResourceRequest::new(250, 1 << 20, 1 << 20).unwrap(),
        Job::AnalyzeStability {
            world: stable_node_world(),
            config: StabilityConfig::new(vec![(-1.0, 1.0), (-1.0, 1.0)]),
        },
    )
    .unwrap();
    assert_eq!(envelope.work.kind, "analyze-stability");
    let worker = Worker::new(
        WorkerConfig::new(ResourceRequest::new(1_000, 2 << 20, 2 << 20).unwrap(), 1024).unwrap(),
        LocalStore::open(&root, StoreConfig::default()).unwrap(),
    )
    .unwrap();
    let output = worker.execute_at(&envelope, &CancellationToken::default(), 2).unwrap();
    let JobOutput::Stability(report) = output else {
        panic!("worker dispatched stability analysis to the wrong executor")
    };
    assert_eq!(report.fixed_points.len(), 1);
    assert_eq!(report.fixed_points[0].classification, Classification::StableNode);
    let _ = std::fs::remove_dir_all(root);
}
