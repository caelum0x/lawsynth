//! Determinism: identical `(Dataset, SdeConfig)` inputs MUST yield a
//! bit-identical `SdeModel` (`f64::to_bits`), and re-simulating the same seeded
//! path MUST reproduce the same data.

mod common;

use common::*;
use lawsynth_sde::{BinRule, SdeConfig, discover_sde};

#[test]
fn identical_inputs_produce_bit_identical_models() {
    let (time, values) = ornstein_uhlenbeck(1.0, 0.5, 0.01, 200_000, 42);
    let dataset = scalar_dataset(time, values);
    let config = SdeConfig::new().with_bins(BinRule::Count(24)).with_min_bin_count(50);

    let first = discover_sde(&dataset, &config).unwrap();
    let second = discover_sde(&dataset, &config).unwrap();

    // Structural equality and, more strictly, bit-identical floats.
    assert_eq!(first, second);
    assert_eq!(model_bits(&first), model_bits(&second));
}

#[test]
fn seeded_path_is_reproducible() {
    let a = ornstein_uhlenbeck(1.0, 0.5, 0.01, 50_000, 7);
    let b = ornstein_uhlenbeck(1.0, 0.5, 0.01, 50_000, 7);
    assert_eq!(
        a.1.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        b.1.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );
}

#[test]
fn different_seeds_produce_different_paths() {
    let a = ornstein_uhlenbeck(1.0, 0.5, 0.01, 50_000, 7);
    let b = ornstein_uhlenbeck(1.0, 0.5, 0.01, 50_000, 8);
    assert_ne!(a.1, b.1);
}

#[test]
fn ensemble_discovery_is_bit_identical() {
    let (time, values) = geometric_brownian_ensemble(0.5, 0.5, 0.4, 4.0, 0.0005, 500, 200, 7);
    let dataset = scalar_dataset(time, values);
    let config = SdeConfig::new()
        .with_bins(BinRule::Count(20))
        .with_min_bin_count(50)
        .with_polynomial_degree(2)
        .with_trajectories(500);
    let first = discover_sde(&dataset, &config).unwrap();
    let second = discover_sde(&dataset, &config).unwrap();
    assert_eq!(model_bits(&first), model_bits(&second));
}
