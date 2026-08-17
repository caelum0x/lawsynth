//! Dependency/causal hypothesis discovery (§8.6): the opt-in pass recovers a
//! known lagged direction, excludes the reverse, and reports its result as a
//! hypothesis contingent on a declared assumption set — never proven causation.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::{CausalAssumption, DiscoveryConfig, discover};

/// A deterministic AR(1) driver `x` and its one-sample lag `y = x[t-1]`.
///
/// `x` is Markov of order one, so `x[t-2]` adds no predictive power for `x[t]`:
/// the reverse Granger direction stays weak. `y` is an exact lag of `x`, so
/// `x[t-1]` fully explains `y[t]`: the forward direction is strong. The AR(1)
/// coefficient keeps the marginal correlation well above the independence floor.
/// A fixed linear-congruential generator supplies deterministic innovations —
/// no RNG crate, no wall clock.
fn lagged_driver_and_response(samples: usize) -> Dataset {
    let x_id = Identifier::new("x").unwrap();
    let y_id = Identifier::new("y").unwrap();

    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    let mut innovation = || {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // Map the top bits into [-0.5, 0.5].
        ((state >> 33) as f64 / (1u64 << 31) as f64) - 0.5
    };

    let mut x = Vec::with_capacity(samples);
    x.push(0.0);
    for _ in 1..samples {
        let previous = *x.last().unwrap();
        x.push(0.6 * previous + innovation());
    }
    // y[t] = x[t - 1]; y[0] seeded to the initial state.
    let mut y = Vec::with_capacity(samples);
    y.push(x[0]);
    for index in 1..samples {
        y.push(x[index - 1]);
    }

    let time = (0..samples).map(|step| step as f64).collect::<Vec<_>>();
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [NumericColumn::new(x_id, x), NumericColumn::new(y_id, y)],
    )
    .unwrap()
}

#[test]
fn hypothesis_recovers_the_forward_lagged_edge_and_excludes_the_reverse() {
    let dataset = lagged_driver_and_response(80);
    let mut config = DiscoveryConfig::new([Identifier::new("y").unwrap()]);
    config.enable_causal_hypothesis();

    let result = discover(&dataset, &config).unwrap();
    let graph = result.dependency_hypothesis.as_ref().expect("hypothesis should be produced");

    assert!(graph.has_edge("x", "y"), "expected forward edge x -> y");
    assert!(!graph.has_edge("y", "x"), "reverse edge y -> x must be excluded");
    assert_eq!(graph.edges().count(), 1);
}

#[test]
fn hypothesis_reports_the_assumptions_it_is_contingent_on() {
    let dataset = lagged_driver_and_response(80);
    let mut config = DiscoveryConfig::new([Identifier::new("y").unwrap()]);
    config.enable_causal_hypothesis();

    let result = discover(&dataset, &config).unwrap();
    let assumptions = result.dependency_assumptions.expect("assumptions accompany the hypothesis");

    // The graph is a candidate structure: it is only a causal reading under an
    // explicitly declared assumption set, mirroring lawsynth-causal's framing.
    assert!(assumptions.contains(&CausalAssumption::Faithfulness));
    assert!(assumptions.contains(&CausalAssumption::CausalSufficiency));
}

#[test]
fn the_default_path_produces_no_causal_hypothesis() {
    let dataset = lagged_driver_and_response(80);
    let result =
        discover(&dataset, &DiscoveryConfig::new([Identifier::new("y").unwrap()])).unwrap();
    assert!(result.dependency_hypothesis.is_none());
    assert!(result.dependency_assumptions.is_none());
}
