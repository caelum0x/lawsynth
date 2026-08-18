//! Statistical honesty: the Kramers–Moyal estimate is noisy on finite data, and
//! a LONGER path tightens it. This test demonstrates that increasing the number
//! of samples reduces the error of the recovered OU drift slope — documenting
//! that the tolerances used elsewhere are a function of path length, not a claim
//! of machine-precision recovery.

mod common;

use common::*;
use lawsynth_sde::{BinRule, SdeConfig, discover_sde};

const THETA: f64 = 1.0;
const SIGMA: f64 = 0.5;

fn drift_slope_error(steps: usize) -> f64 {
    let (time, values) = ornstein_uhlenbeck(THETA, SIGMA, 0.01, steps, 2024);
    let dataset = scalar_dataset(time, values);
    let config = SdeConfig::new()
        .with_bins(BinRule::Count(20))
        .with_min_bin_count(20)
        .with_polynomial_degree(3);
    let model = discover_sde(&dataset, &config).unwrap();
    let slope = model.state(&x()).unwrap().drift_law.coefficient_of_power(1);
    (slope - (-THETA)).abs()
}

#[test]
fn more_samples_reduce_the_drift_error() {
    let short = drift_slope_error(20_000);
    let long = drift_slope_error(400_000);
    assert!(
        long < short,
        "expected the longer path (err={long}) to beat the shorter path (err={short})"
    );
    // The long path should land the slope comfortably close to the truth.
    assert!(long < 0.05, "long-path drift error {long} unexpectedly large");
}
