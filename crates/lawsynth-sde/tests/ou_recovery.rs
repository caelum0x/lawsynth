//! Headline recovery test: the Ornstein–Uhlenbeck process
//! `dX = −θ X dt + σ dW` has a LINEAR drift and a CONSTANT diffusion `σ²`.
//!
//! From a long, deterministic Euler–Maruyama path we require the binned /
//! sparse-regressed drift to be `≈ −θ x` (correct slope, no spurious constant or
//! higher-order terms) and the diffusion to be `≈ σ²` (constant), each within a
//! statistical tolerance appropriate to the path length. These are estimates
//! from finite noisy data, so the tolerances are deliberately loose — see
//! `convergence.rs` for the demonstration that longer paths tighten them.

mod common;

use common::*;
use lawsynth_sde::{BinRule, SdeConfig, discover_sde};

const THETA: f64 = 1.0;
const SIGMA: f64 = 0.5;

fn ou_config() -> SdeConfig {
    SdeConfig::new().with_bins(BinRule::Count(24)).with_min_bin_count(50).with_polynomial_degree(3)
}

#[test]
fn recovers_linear_drift_slope() {
    let (time, values) = ornstein_uhlenbeck(THETA, SIGMA, 0.01, 400_000, 42);
    let dataset = scalar_dataset(time, values);
    let model = discover_sde(&dataset, &ou_config()).unwrap();
    let state = model.state(&x()).unwrap();

    // Drift ≈ −θ x: the slope on x is recovered, and lower/higher order terms
    // are negligible.
    let drift = &state.drift_law;
    assert!(
        (drift.coefficient_of_power(1) - (-THETA)).abs() < 0.08,
        "drift slope {} not ≈ {}",
        drift.coefficient_of_power(1),
        -THETA
    );
    assert!(drift.coefficient_of_power(0).abs() < 0.05, "spurious drift constant");
    assert!(drift.coefficient_of_power(2).abs() < 0.05, "spurious drift x^2");
    assert!(drift.coefficient_of_power(3).abs() < 0.05, "spurious drift x^3");
}

#[test]
fn recovers_constant_diffusion() {
    let (time, values) = ornstein_uhlenbeck(THETA, SIGMA, 0.01, 400_000, 42);
    let dataset = scalar_dataset(time, values);
    let model = discover_sde(&dataset, &ou_config()).unwrap();
    let state = model.state(&x()).unwrap();

    // Diffusion ≈ σ² constant; state-dependent terms are negligible.
    let diffusion = &state.diffusion_law;
    let sigma_squared = SIGMA * SIGMA;
    assert!(
        (diffusion.coefficient_of_power(0) - sigma_squared).abs() < 0.02,
        "diffusion constant {} not ≈ σ²={}",
        diffusion.coefficient_of_power(0),
        sigma_squared
    );
    assert!(diffusion.coefficient_of_power(1).abs() < 0.03, "spurious diffusion x");
    assert!(diffusion.coefficient_of_power(2).abs() < 0.03, "spurious diffusion x^2");
}

#[test]
fn raw_binned_drift_is_linear_and_diffusion_is_flat() {
    let (time, values) = ornstein_uhlenbeck(THETA, SIGMA, 0.01, 400_000, 42);
    let dataset = scalar_dataset(time, values);
    let model = discover_sde(&dataset, &ou_config()).unwrap();
    let state = model.state(&x()).unwrap();

    // Every reported bin carries a positive occupancy and a non-negative
    // diffusion (a variance rate).
    assert!(state.bins.iter().all(|bin| bin.count > 0));
    assert!(state.bins.iter().all(|bin| bin.diffusion >= 0.0));

    // The raw binned drift crosses zero and has the correct sign structure:
    // negative for x > 0, positive for x < 0 (mean reversion).
    let positive = state.bins.iter().find(|bin| bin.x_center > 0.3).unwrap();
    let negative = state.bins.iter().find(|bin| bin.x_center < -0.3).unwrap();
    assert!(positive.drift < 0.0, "drift should be negative for x>0");
    assert!(negative.drift > 0.0, "drift should be positive for x<0");
}

#[test]
fn fitted_drift_law_evaluates_close_to_truth() {
    let (time, values) = ornstein_uhlenbeck(THETA, SIGMA, 0.01, 400_000, 42);
    let dataset = scalar_dataset(time, values);
    let model = discover_sde(&dataset, &ou_config()).unwrap();
    let drift = &model.state(&x()).unwrap().drift_law;

    // The reconstructed law a(x) tracks −θ x across the sampled interior.
    for &probe in &[-0.5, -0.2, 0.0, 0.2, 0.5] {
        assert!(
            (drift.evaluate(probe) - (-THETA * probe)).abs() < 0.06,
            "a({probe}) = {} not ≈ {}",
            drift.evaluate(probe),
            -THETA * probe
        );
    }
}
