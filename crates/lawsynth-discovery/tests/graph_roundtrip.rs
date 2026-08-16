use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::infer_lagged_dependencies;

#[test]
fn lagged_dependency_inference_is_deterministic_and_ignores_constant_columns() {
    let x = Identifier::new("x").unwrap();
    let y = Identifier::new("y").unwrap();
    let constant = Identifier::new("constant").unwrap();
    let dataset = Dataset::new(
        TimeAxis::new((0..7).map(|value| value as f64).collect()).unwrap(),
        [
            NumericColumn::new(x, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            NumericColumn::new(y, vec![9.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
            NumericColumn::new(constant, vec![1.0; 7]),
        ],
    )
    .unwrap();
    let first = infer_lagged_dependencies(&dataset, 2, 0.99).unwrap();
    assert_eq!(first, infer_lagged_dependencies(&dataset, 2, 0.99).unwrap());
    assert_eq!(first.edges.len(), 1);
}
