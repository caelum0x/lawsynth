//! Determinism contract: identical inputs produce bit-identical results.

mod support;

use lawsynth_implicit::{ImplicitConfig, implicit_discover};
use support::{dataset_x, integrate, michaelis_menten};

#[test]
fn identical_inputs_yield_bit_identical_results() {
    let (time, xs) = integrate(michaelis_menten(1.2, 0.4), 2.0, 0.01, 300);
    let dataset = dataset_x(time, xs);
    let config = ImplicitConfig { degree: 2, ..Default::default() };

    let first = implicit_discover(&dataset, &config).unwrap();
    let second = implicit_discover(&dataset, &config).unwrap();

    // Structural equality, then exact bitwise equality of the formatted output.
    assert_eq!(first, second);
    assert_eq!(format!("{first:?}"), format!("{second:?}"));

    // The reported dataset fingerprint is stable across replays.
    assert_eq!(first.diagnostics.dataset_fingerprint, second.diagnostics.dataset_fingerprint);
}

#[test]
fn candidate_scores_are_recorded_for_every_column() {
    let (time, xs) = integrate(michaelis_menten(1.0, 0.5), 2.0, 0.01, 200);
    let dataset = dataset_x(time, xs);
    let config = ImplicitConfig { degree: 1, ..Default::default() };

    let result = implicit_discover(&dataset, &config).unwrap();
    // Library at degree 1 with a constant: [1, x, dx, x*dx] -> 4 candidates.
    assert_eq!(result.diagnostics.library_size, 4);
    assert_eq!(result.diagnostics.candidates_evaluated, 4);

    // The constant column `1` normalises to a non-trivial LHS but has no sparse
    // consistent expansion, so it is rejected: it must NOT be the winner and
    // its relative residual must dominate the chosen relation's.
    assert_ne!(result.relation.lhs_name, "1");
    let constant = result
        .diagnostics
        .candidate_scores
        .iter()
        .find(|score| score.lhs_name == "1")
        .expect("constant column scored");
    assert!(
        constant.relative_residual > result.relation.relative_residual,
        "constant LHS residual {} did not exceed winner {}",
        constant.relative_residual,
        result.relation.relative_residual
    );
    // The winning relation genuinely involves the derivative (it is dynamics,
    // not a static algebraic constraint among the states).
    assert!(result.relation.terms.iter().any(|term| term.term.involves_derivative));
}
