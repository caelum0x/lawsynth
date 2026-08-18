//! Nonlinear drift: the double-well process `dX = (X − X³) dt + σ dW` has a
//! CUBIC drift `x − x³` and a constant diffusion `σ²`.
//!
//! The sparse fit must recover the cubic structure — a positive linear term and
//! a negative cubic term — from a long deterministic path that the well confines
//! to a bounded interval (so the state space is well sampled).

mod common;

use common::*;
use lawsynth_sde::{BinRule, SdeConfig, discover_sde};

const SIGMA: f64 = 0.5;

fn double_well_model() -> lawsynth_sde::SdeModel {
    let (time, values) = double_well(SIGMA, 0.005, 800_000, 99);
    let dataset = scalar_dataset(time, values);
    let config = SdeConfig::new()
        .with_bins(BinRule::Count(30))
        .with_min_bin_count(50)
        .with_polynomial_degree(3);
    discover_sde(&dataset, &config).unwrap()
}

#[test]
fn recovers_cubic_drift_structure() {
    let model = double_well_model();
    let drift = &model.state(&x()).unwrap().drift_law;

    // Drift ≈ x − x³: positive linear term ≈ +1, negative cubic term ≈ −1.
    assert!(
        (drift.coefficient_of_power(1) - 1.0).abs() < 0.1,
        "drift x {} not ≈ 1",
        drift.coefficient_of_power(1)
    );
    assert!(
        (drift.coefficient_of_power(3) - (-1.0)).abs() < 0.1,
        "drift x^3 {} not ≈ -1",
        drift.coefficient_of_power(3)
    );
    assert!(drift.coefficient_of_power(0).abs() < 0.05, "spurious drift constant");
    assert!(drift.coefficient_of_power(2).abs() < 0.05, "spurious drift x^2");
}

#[test]
fn cubic_term_is_selected_by_the_sparse_fit() {
    let model = double_well_model();
    let drift = &model.state(&x()).unwrap().drift_law;
    // The x³ term survives thresholding — the nonlinearity is genuinely selected.
    let active_powers = drift.active_terms().map(|term| term.power).collect::<Vec<_>>();
    assert!(active_powers.contains(&1), "linear term must be active");
    assert!(active_powers.contains(&3), "cubic term must be active");
}

#[test]
fn recovers_constant_diffusion_under_nonlinear_drift() {
    let model = double_well_model();
    let diffusion = &model.state(&x()).unwrap().diffusion_law;
    assert!(
        (diffusion.coefficient_of_power(0) - SIGMA * SIGMA).abs() < 0.02,
        "diffusion constant {} not ≈ σ²={}",
        diffusion.coefficient_of_power(0),
        SIGMA * SIGMA
    );
}

#[test]
fn drift_has_two_stable_wells() {
    let model = double_well_model();
    let drift = &model.state(&x()).unwrap().drift_law;
    // The recovered a(x) = x − x³ pushes toward ±1: negative pull at x = 1.5,
    // positive push at x = −1.5.
    assert!(drift.evaluate(1.5) < 0.0);
    assert!(drift.evaluate(-1.5) > 0.0);
}
