use std::collections::BTreeMap;

use lawsynth_core::Identifier;

#[path = "../src/py_dataset.rs"]
mod py_dataset;

use py_dataset::dataset_from_columns;

#[test]
fn constructs_an_aligned_dataset_with_deterministic_schema() {
    let dataset = dataset_from_columns(
        vec![0.0, 0.25, 1.0],
        BTreeMap::from([
            ("velocity".to_owned(), vec![0.0, 1.0, 2.0]),
            ("position".to_owned(), vec![2.0, 2.25, 3.0]),
        ]),
    )
    .expect("aligned Python values should form a dataset");

    assert_eq!(dataset.time().values(), &[0.0, 0.25, 1.0]);
    assert_eq!(dataset.schema().columns.len(), 2);
    assert_eq!(
        dataset.columns()[&Identifier::new("position").expect("valid id")].values,
        vec![2.0, 2.25, 3.0]
    );
    assert_eq!(
        dataset.columns().keys().map(ToString::to_string).collect::<Vec<_>>(),
        vec!["position", "velocity"]
    );
}

#[test]
fn preserves_dataset_validation_at_the_python_boundary() {
    let mismatch =
        dataset_from_columns(vec![0.0, 1.0], BTreeMap::from([("x".to_owned(), vec![1.0])]));
    assert!(mismatch.expect_err("column length mismatch must be rejected").contains("expected 2"));

    let malformed_name =
        dataset_from_columns(vec![0.0], BTreeMap::from([("not a column".to_owned(), vec![1.0])]));
    assert!(malformed_name.is_err());
}
