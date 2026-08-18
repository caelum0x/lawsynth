use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_differentiate::differentiate_dataset;

#[test]
fn differentiates_all_aligned_dataset_columns_and_preserves_metadata() {
    let dataset = Dataset::new(
        TimeAxis::new(vec![0.0, 0.5, 2.0, 3.0]).unwrap(),
        [
            NumericColumn::new(Identifier::new("position").unwrap(), vec![0.0, 0.25, 4.0, 9.0])
                .with_unit("m"),
            NumericColumn::new(Identifier::new("velocity").unwrap(), vec![1.0, 2.0, 5.0, 7.0]),
        ],
    )
    .unwrap();

    let derivative = differentiate_dataset(&dataset).unwrap();
    assert_eq!(derivative.time(), dataset.time());
    assert_eq!(derivative.columns()[&Identifier::new("position").unwrap()].unit, Some("m".into()));
    assert_eq!(
        derivative.columns()[&Identifier::new("position").unwrap()].values,
        vec![0.5, 1.0, 4.0, 5.0]
    );
    assert_eq!(
        derivative.columns()[&Identifier::new("velocity").unwrap()].values,
        vec![2.0, 2.0, 2.0, 2.0]
    );
}
