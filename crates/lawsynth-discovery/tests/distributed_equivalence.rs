//! P10 distributed-discovery equivalence: `discover_partitioned` must return a
//! result bit-identical to single-node `discover` for every partition count.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::{DiscoveryConfig, SparseMethod, discover, discover_partitioned};

/// Three coupled, smooth state variables sampled densely enough to exercise a
/// wide polynomial + trigonometric + rational feature library.
fn synthetic_dataset() -> Dataset {
    let x = Identifier::new("x").unwrap();
    let y = Identifier::new("y").unwrap();
    let z = Identifier::new("z").unwrap();
    let time = (0..400).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
    Dataset::new(
        TimeAxis::new(time.clone()).unwrap(),
        [
            NumericColumn::new(x, time.iter().map(|t| (2.0 * t).sin() + 0.5 * t).collect()),
            NumericColumn::new(y, time.iter().map(|t| (1.3 * t).cos() - 0.2 * t * t).collect()),
            NumericColumn::new(z, time.iter().map(|t| 0.3 * t * t - t + (0.7 * t).sin()).collect()),
        ],
    )
    .unwrap()
}

fn rich_config() -> DiscoveryConfig {
    let states =
        ["x", "y", "z"].map(|name| Identifier::new(name).unwrap()).into_iter().collect::<Vec<_>>();
    let mut config = DiscoveryConfig::new(states);
    config.polynomial_degree = 3;
    config.include_trigonometric = true;
    config.include_rational = true;
    config.sparse.threshold = 0.05;
    config.sparse_method = SparseMethod::Stlsq;
    // Exercise the bootstrap + causal + regime passes too: they run identically
    // on both paths and must not be perturbed by feature partitioning.
    config.bootstrap =
        Some(lawsynth_stats::BootstrapConfig { replicates: 6, block_size: 8, seed: 11 });
    config.enable_causal_hypothesis();
    config.enable_regimes();
    config
}

#[test]
fn partitioned_discovery_is_identical_to_single_node() {
    let dataset = synthetic_dataset();
    let config = rich_config();
    let baseline = discover(&dataset, &config).expect("single-node discovery");

    for partitions in [1usize, 2, 3, 7] {
        let result = discover_partitioned(&dataset, &config, partitions).unwrap_or_else(|error| {
            panic!("partitioned discovery failed at p={partitions}: {error}")
        });

        // Whole-result equality: same candidates, frontier, regimes, hypothesis,
        // profile and preprocessing provenance.
        assert_eq!(
            result, baseline,
            "distributed result diverged from single-node at partitions={partitions}"
        );

        // Explicit, per-law bit-for-bit check of the discovered coefficients so a
        // regression here is unambiguous and not hidden behind derived PartialEq.
        assert_eq!(result.candidates.len(), baseline.candidates.len());
        for (candidate, expected) in result.candidates.iter().zip(&baseline.candidates) {
            assert_eq!(candidate.world, expected.world, "world differs at p={partitions}");
            assert_eq!(
                candidate.metrics.mean_squared_error.to_bits(),
                expected.metrics.mean_squared_error.to_bits(),
                "MSE differs at p={partitions}"
            );
            assert_eq!(
                candidate.metrics.complexity, expected.metrics.complexity,
                "complexity differs at p={partitions}"
            );
        }
        assert_eq!(result.frontier, baseline.frontier, "frontier differs at p={partitions}");
    }
}
