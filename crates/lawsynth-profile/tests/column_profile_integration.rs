use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_profile::{distribution, profile, quality_flags};

#[test]
fn dataset_profile_agrees_with_column_distribution_and_quality_diagnostics() {
    let id = Identifier::new("signal").unwrap();
    let values = vec![1.0, 2.0, 3.0, 4.0, 100.0];
    let dataset = Dataset::new(
        TimeAxis::new(vec![0.0, 1.0, 2.0, 3.0, 4.0]).unwrap(),
        [NumericColumn::new(id.clone(), values.clone())],
    )
    .unwrap();
    let result = profile(&dataset).unwrap();
    let profile_column = &result.columns[&id];
    assert_eq!(profile_column.minimum, distribution(&values).unwrap().minimum);
    assert_eq!(profile_column.maximum, distribution(&values).unwrap().maximum);
    assert_eq!(result.quality[&id], quality_flags(&values).unwrap());
    assert_eq!(result.quality[&id].outlier_indices, vec![4]);
}
