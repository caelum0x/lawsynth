//! The job execution core.
//!
//! This is the single place that maps a typed [`Job`] onto the underlying
//! discovery and simulation engines and honours cooperative cancellation. It is
//! deliberately free of admission, checkpointing, and transport concerns so the
//! same logic can be exercised directly in tests and reused by [`crate::Worker`].

use lawsynth_runner::CancellationToken;

use crate::{Job, JobOutput, WorkerError};

/// Runs one typed job to completion, mapping engine errors into [`WorkerError`].
///
/// Cancellation is checked once after the engine returns: the engines are
/// synchronous, so a token flipped during execution is observed on the way out
/// and turns any result into [`WorkerError::Cancelled`].
pub(crate) fn run(job: &Job, cancellation: &CancellationToken) -> Result<JobOutput, WorkerError> {
    let result = match job {
        Job::Discover { dataset, config } => lawsynth_discovery::discover(dataset, config)
            .map(JobOutput::Discovery)
            .map_err(WorkerError::from),
        Job::Simulate { world, config, request } => lawsynth_sim::simulate(world, *config, request)
            .map(JobOutput::Simulation)
            .map_err(WorkerError::from),
        Job::AnalyzeStability { world, config } => {
            // Read the world's laws as an autonomous vector field `ẋ = f(x)`,
            // mirroring the CLI's `lawsynth stability` derivation: one
            // `(state, right-hand side)` pair per continuous law, in state order.
            let states = world.state_ids().cloned().collect::<Vec<_>>();
            let fields = world
                .laws()
                .iter()
                .map(|(target, law)| (target.clone(), law.expression.clone()))
                .collect::<Vec<_>>();
            lawsynth_stability::analyze_stability(&fields, &states, config)
                .map(JobOutput::Stability)
                .map_err(WorkerError::from)
        }
    };
    match cancellation.reason() {
        Some(reason) => Err(WorkerError::Cancelled(reason)),
        None => result,
    }
}

/// A short, human-readable summary of a completed job's output, recorded in the
/// terminal checkpoint's detail field.
pub(crate) fn output_summary(output: &JobOutput) -> String {
    match output {
        JobOutput::Discovery(result) => {
            format!("discovery completed with {} Pareto candidate(s)", result.candidates.len())
        }
        JobOutput::Simulation(trajectory) => {
            format!("simulation completed with {} sample(s)", trajectory.samples())
        }
        JobOutput::Stability(report) => {
            format!(
                "stability analysis found {} fixed point(s), {} converged seed(s)",
                report.fixed_points.len(),
                report.seeds_converged,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_expr::{Expr, UnaryOperator};
    use lawsynth_runner::CancellationToken;
    use lawsynth_stability::{Classification, StabilityConfig};
    use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

    use super::{output_summary, run};
    use crate::{Job, JobOutput, WorkerError};

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    /// A linear stable node at the origin: `x' = -x`, `y' = -2y`.
    fn stable_node_world() -> World {
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            [],
            [
                ContinuousLaw::new(
                    id("x"),
                    Expr::unary(UnaryOperator::Negate, Expr::symbol(id("x"))),
                ),
                ContinuousLaw::new(
                    id("y"),
                    Expr::product(Expr::constant(-2.0), Expr::symbol(id("y"))),
                ),
            ],
        )
        .unwrap()
    }

    fn stability_job() -> Job {
        Job::AnalyzeStability {
            world: stable_node_world(),
            config: StabilityConfig::new(vec![(-1.0, 1.0), (-1.0, 1.0)]),
        }
    }

    #[test]
    fn runs_stability_and_classifies_origin_as_stable_node() {
        let output = run(&stability_job(), &CancellationToken::default()).unwrap();
        let JobOutput::Stability(report) = output else {
            panic!("stability job produced the wrong output variant")
        };
        assert_eq!(report.fixed_points.len(), 1);
        assert_eq!(report.fixed_points[0].classification, Classification::StableNode);
        assert!(report.seeds_converged >= 1);
    }

    #[test]
    fn cancellation_turns_a_completed_stability_run_into_cancelled() {
        let cancellation = CancellationToken::default();
        cancellation.cancel("operator stopped job").unwrap();
        let error = run(&stability_job(), &cancellation).unwrap_err();
        assert!(matches!(error, WorkerError::Cancelled(_)));
    }

    #[test]
    fn summary_reports_fixed_points_and_converged_seeds() {
        let output = run(&stability_job(), &CancellationToken::default()).unwrap();
        let summary = output_summary(&output);
        assert!(summary.contains("stability analysis found 1 fixed point(s)"));
        assert!(summary.contains("converged seed(s)"));
    }
}
