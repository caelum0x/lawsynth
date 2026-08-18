//! State-dependent diffusion: geometric Brownian motion
//! `dX = μ X dt + σ X dW` has drift `μ x` and diffusion `b²(x) = σ² x²`.
//!
//! Recovering the QUADRATIC diffusion `σ² x²` is the headline — it proves the
//! estimator handles state-dependent (multiplicative) diffusion, not just an
//! additive constant. GBM has no stationary distribution, so a single long path
//! either explodes or collapses; we therefore sample an ENSEMBLE of short
//! trajectories launched from initial conditions spread across the state space
//! (see `geometric_brownian_ensemble`) and pass it with
//! `SdeConfig::with_trajectories`, which forms increments only within a path.

mod common;

use common::*;
use lawsynth_sde::{BinRule, SdeConfig, discover_sde};

const MU: f64 = 0.5;
const SIGMA: f64 = 0.5;
const TRAJECTORIES: usize = 2000;
const SEGMENT_LEN: usize = 250;

fn gbm_model() -> lawsynth_sde::SdeModel {
    let (time, values) =
        geometric_brownian_ensemble(MU, SIGMA, 0.4, 4.0, 0.0005, TRAJECTORIES, SEGMENT_LEN, 7);
    let dataset = scalar_dataset(time, values);
    // Degree 2 is the right library: drift is `μ x` and diffusion is `σ² x²`, so
    // no cubic term is needed and dropping it avoids over-fitting a noisy tail.
    let config = SdeConfig::new()
        .with_bins(BinRule::Count(24))
        .with_min_bin_count(50)
        .with_polynomial_degree(2)
        .with_trajectories(TRAJECTORIES);
    discover_sde(&dataset, &config).unwrap()
}

#[test]
fn recovers_quadratic_state_dependent_diffusion() {
    let model = gbm_model();
    let diffusion = &model.state(&x()).unwrap().diffusion_law;
    let sigma_squared = SIGMA * SIGMA;

    // b²(x) ≈ σ² x²: the x² term carries σ², with negligible constant / linear.
    assert!(
        (diffusion.coefficient_of_power(2) - sigma_squared).abs() < 0.04,
        "diffusion x^2 {} not ≈ σ²={}",
        diffusion.coefficient_of_power(2),
        sigma_squared
    );
    assert!(diffusion.coefficient_of_power(0).abs() < 0.06, "spurious diffusion constant");
    assert!(diffusion.coefficient_of_power(1).abs() < 0.06, "spurious diffusion x");
}

#[test]
fn recovers_linear_drift() {
    let model = gbm_model();
    let drift = &model.state(&x()).unwrap().drift_law;

    // Drift ≈ μ x. GBM drift is a small signal buried under multiplicative noise,
    // so the tolerance is looser than the (much cleaner) diffusion recovery.
    assert!(
        (drift.coefficient_of_power(1) - MU).abs() < 0.12,
        "drift slope {} not ≈ μ={}",
        drift.coefficient_of_power(1),
        MU
    );
    assert!(drift.coefficient_of_power(0).abs() < 0.15, "spurious drift constant");
    assert!(drift.coefficient_of_power(2).abs() < 0.05, "spurious drift x^2");
}

#[test]
fn diffusion_grows_with_state() {
    let model = gbm_model();
    let diffusion = &model.state(&x()).unwrap().diffusion_law;
    // A genuinely state-dependent diffusion: larger |x| ⇒ larger b²(x).
    assert!(diffusion.evaluate(3.0) > diffusion.evaluate(1.0));
    assert!(diffusion.evaluate(1.0) > diffusion.evaluate(0.5));
}
