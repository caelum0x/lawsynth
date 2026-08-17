//! Joint parameter refinement (§8.5): the opt-in pass must never worsen and
//! typically improves the trajectory fit, and it must be deterministic.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::{DiscoveryConfig, discover};

/// Exponential growth `x(t) = exp(2t)` sampled on a uniform grid. The sparse fit
/// recovers `dx/dt ~= 2 x` from finite-difference derivatives, but explicit-Euler
/// simulation of that law undershoots the true trajectory, so refining the
/// coefficient against the observed path measurably reduces trajectory error.
fn exponential_growth() -> (Dataset, Identifier) {
    let x = Identifier::new("x").unwrap();
    let time = (0..101).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
    let values = time.iter().map(|time| (2.0 * time).exp()).collect::<Vec<_>>();
    let dataset =
        Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(x.clone(), values)])
            .unwrap();
    (dataset, x)
}

#[test]
fn refinement_improves_trajectory_mse_without_ever_worsening_it() {
    let (dataset, _x) = exponential_growth();
    let mut config = DiscoveryConfig::new([Identifier::new("x").unwrap()]);
    config.enable_refinement();

    let result = discover(&dataset, &config).unwrap();
    let candidate = &result.candidates[0];
    let refinement = candidate.refinement.as_ref().expect("refinement pass should run");

    // The optimizer starts from the discovered constants and only accepts
    // improvements, so the refined error can never exceed the initial one.
    assert!(refinement.mse_after <= refinement.mse_before);
    // On this synthetic system the Euler-simulated fit is genuinely improvable.
    assert!(
        refinement.mse_after < refinement.mse_before,
        "expected strict improvement, got before={} after={}",
        refinement.mse_before,
        refinement.mse_after
    );
    assert!(refinement.improvement() > 0.0);
    assert!(refinement.iterations > 0);
}

#[test]
fn refinement_is_deterministic_across_runs() {
    let (dataset, _x) = exponential_growth();
    let mut config = DiscoveryConfig::new([Identifier::new("x").unwrap()]);
    config.enable_refinement();

    let first = discover(&dataset, &config).unwrap();
    let second = discover(&dataset, &config).unwrap();

    assert_eq!(first.candidates[0].refinement, second.candidates[0].refinement);
    assert_eq!(first.candidates[0].world, second.candidates[0].world);
}

#[test]
fn refinement_leaves_the_default_metrics_untouched_but_updates_the_world() {
    let (dataset, x) = exponential_growth();
    let baseline = discover(&dataset, &DiscoveryConfig::new([x.clone()])).unwrap();

    let mut config = DiscoveryConfig::new([x.clone()]);
    config.enable_refinement();
    let refined = discover(&dataset, &config).unwrap();

    // The reported derivative-fit metric is unchanged (refinement optimizes the
    // trajectory objective, not the derivative residual), while the refined world
    // carries different constants when the trajectory fit strictly improves.
    assert_eq!(
        baseline.candidates[0].metrics.mean_squared_error,
        refined.candidates[0].metrics.mean_squared_error
    );
    assert!(baseline.candidates[0].refinement.is_none());
    assert_ne!(baseline.candidates[0].world, refined.candidates[0].world);
}
