use lawsynth_core::{Identifier, ResourceLimitError, ResourceLimits};
use lawsynth_data::{Dataset, DatasetConfig, NumericColumn, TimeAxis};

fn dataset() -> Dataset {
    Dataset::new(
        TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
        [NumericColumn::new(Identifier::new("x").unwrap(), vec![1.0, 2.0, 3.0])],
    )
    .unwrap()
}

#[test]
fn opt_in_resource_configuration_bounds_dataset_shapes() {
    let config = DatasetConfig {
        resource_limits: ResourceLimits {
            max_samples: 2,
            max_columns: 1,
            max_features: 10,
            max_candidates: 10,
        },
    };
    assert_eq!(
        config.validate(&dataset()),
        Err(ResourceLimitError::Exceeded { resource: "samples", actual: 3, limit: 2 })
    );
}
