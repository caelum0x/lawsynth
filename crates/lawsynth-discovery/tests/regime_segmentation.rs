//! Opt-in regime segmentation and default-path frontier regression coverage.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::{DiscoveryConfig, discover};

/// A single-jump, piecewise-constant series: fifteen samples at 0.0 followed by
/// fifteen at 10.0. Exact PELT segmentation must recover precisely two regimes.
fn piecewise_constant_dataset() -> (Dataset, Identifier) {
    let x = Identifier::new("x").unwrap();
    let time = (0..30).map(|step| step as f64).collect::<Vec<_>>();
    let values = (0..30).map(|step| if step < 15 { 0.0 } else { 10.0 }).collect::<Vec<_>>();
    let dataset =
        Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(x.clone(), values)])
            .unwrap();
    (dataset, x)
}

#[test]
fn regimes_are_absent_on_the_default_path() {
    let (dataset, x) = piecewise_constant_dataset();
    let config = DiscoveryConfig::new([x]);
    let result = discover(&dataset, &config).unwrap();
    assert!(result.regimes.is_none());
    // The frontier is always exposed and references only real candidate indices.
    assert!(!result.frontier.is_empty());
    assert!(result.frontier.iter().all(|index| *index < result.candidates.len()));
    assert_eq!(result.pareto_frontier().len(), result.frontier.len());
}

#[test]
fn enabling_regimes_segments_the_primary_state() {
    let (dataset, x) = piecewise_constant_dataset();
    let mut config = DiscoveryConfig::new([x]);
    config.polynomial_degree = 1;
    config.enable_regimes();
    let result = discover(&dataset, &config).unwrap();
    let segmentation = result.regimes.expect("regime pass should record segments");
    assert_eq!(segmentation.segments.len(), 2);
    assert_eq!(segmentation.change_points(), vec![15]);
}
