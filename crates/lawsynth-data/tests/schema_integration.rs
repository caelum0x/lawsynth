use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

#[test]
fn schema_is_independent_of_input_column_order() {
    let dataset = Dataset::new(
        TimeAxis::new(vec![0.0, 1.0]).unwrap(),
        [
            NumericColumn::new(Identifier::new("z").unwrap(), vec![3.0, 4.0]),
            NumericColumn::new(Identifier::new("a").unwrap(), vec![1.0, 2.0]),
        ],
    )
    .unwrap();
    assert_eq!(
        dataset
            .schema()
            .columns
            .iter()
            .map(Identifier::as_str)
            .collect::<Vec<_>>(),
        ["a", "z"]
    );
}
