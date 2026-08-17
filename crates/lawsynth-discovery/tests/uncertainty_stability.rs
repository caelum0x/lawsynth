//! Bootstrap uncertainty is attached to candidates as a deterministic
//! selection-stability score, opt-in via the bootstrap configuration.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::{DiscoveryConfig, discover};
use lawsynth_stats::BootstrapConfig;

fn growth_dataset() -> (Dataset, Identifier) {
    let x = Identifier::new("x").unwrap();
    let time = (0..101).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
    let values = time.iter().map(|time| (2.0 * time).exp()).collect::<Vec<_>>();
    let dataset =
        Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(x.clone(), values)])
            .unwrap();
    (dataset, x)
}

#[test]
fn default_path_leaves_stability_unset() {
    let (dataset, x) = growth_dataset();
    let config = DiscoveryConfig::new([x]);
    let result = discover(&dataset, &config).unwrap();
    assert!(result.candidates[0].stability.is_none());
    assert!(result.candidates[0].bootstrap_mse.is_none());
}

#[test]
fn bootstrap_attaches_a_deterministic_stability_score() {
    let (dataset, x) = growth_dataset();
    let mut config = DiscoveryConfig::new([x]);
    config.bootstrap = Some(BootstrapConfig { replicates: 8, block_size: 4, seed: 11 });

    let first = discover(&dataset, &config).unwrap();
    let stability =
        first.candidates[0].stability.expect("bootstrap should attach a stability score");
    assert!(first.candidates[0].bootstrap_mse.is_some());
    assert!((0.0..=1.0).contains(&stability));

    // Re-running with the identical seeded configuration reproduces the score
    // bit-for-bit, proving the stability summary is deterministic.
    let second = discover(&dataset, &config).unwrap();
    assert_eq!(second.candidates[0].stability, Some(stability));

    // The stability score also feeds the multi-objective score vector.
    assert_eq!(first.candidates[0].score().stability, stability);
}
