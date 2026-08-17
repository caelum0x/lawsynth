//! Pareto frontier dominance over hand-constructed discovery candidates.

use lawsynth_core::Identifier;
use lawsynth_discovery::{DiscoveryCandidate, pareto_frontier};
use lawsynth_expr::Expr;
use lawsynth_score::CandidateMetrics;
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

/// Builds a candidate with a trivial shared world so only the score vector
/// (error, complexity, stability) drives the Pareto comparison.
fn candidate(error: f64, complexity: usize, stability: Option<f64>) -> DiscoveryCandidate {
    let x = Identifier::new("x").unwrap();
    let world = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(x, Expr::constant(0.0))],
    )
    .unwrap();
    DiscoveryCandidate {
        world,
        metrics: CandidateMetrics { mean_squared_error: error, complexity },
        bootstrap_mse: None,
        stability,
        refinement: None,
    }
}

#[test]
fn dominated_candidate_is_excluded_and_incomparable_ones_are_retained() {
    let candidates = vec![
        candidate(0.1, 2, Some(0.9)), // best fit and stability
        candidate(0.5, 4, Some(0.9)), // strictly dominated by index 0
        candidate(0.4, 1, Some(0.5)), // incomparable: lowest complexity
    ];
    // Index 1 is worse on every axis than index 0, so it is removed. Index 2
    // trades higher error for the lowest complexity and stays on the frontier.
    assert_eq!(pareto_frontier(&candidates), vec![0, 2]);
}

#[test]
fn equal_candidates_are_both_kept() {
    let candidates = vec![candidate(0.2, 3, Some(0.7)), candidate(0.2, 3, Some(0.7))];
    assert_eq!(pareto_frontier(&candidates), vec![0, 1]);
}

#[test]
fn higher_stability_dominates_when_fit_and_complexity_tie() {
    let candidates = vec![candidate(0.3, 3, Some(0.95)), candidate(0.3, 3, Some(0.40))];
    assert_eq!(pareto_frontier(&candidates), vec![0]);
}
