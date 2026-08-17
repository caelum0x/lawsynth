//! Regression guard: with none of the new opt-in flags set, `discover` must
//! behave exactly as before — same candidates, same world, and all new fields
//! left empty.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::{DiscoveryConfig, discover};
use lawsynth_expr::print;

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
fn default_discovery_is_unchanged_and_leaves_new_fields_empty() {
    let (dataset, x) = exponential_growth();
    let result = discover(&dataset, &DiscoveryConfig::new([x.clone()])).unwrap();

    // Exactly the pre-existing single sparse candidate, unchanged law.
    assert_eq!(result.candidates.len(), 1);
    let candidate = &result.candidates[0];
    let printed = print(&candidate.world.laws()[&x].expression);
    assert!(printed.contains("2.000"), "unexpected default law: {printed}");

    // None of the additive §8.5 / §8.6 machinery is engaged on the default path.
    assert!(candidate.refinement.is_none());
    assert!(result.dependency_hypothesis.is_none());
    assert!(result.dependency_assumptions.is_none());
    assert!(result.regimes.is_none());
}

#[test]
fn default_discovery_is_byte_identical_across_runs() {
    let (dataset, x) = exponential_growth();
    let config = DiscoveryConfig::new([x.clone()]);

    let first = discover(&dataset, &config).unwrap();
    let second = discover(&dataset, &config).unwrap();

    // Full structural equality of the default result, run to run.
    assert_eq!(first, second);
    assert_eq!(
        print(&first.candidates[0].world.laws()[&x].expression),
        print(&second.candidates[0].world.laws()[&x].expression)
    );
}
