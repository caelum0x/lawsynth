use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

#[test]
fn fingerprints_change_when_values_or_unit_metadata_change() {
    let make = |values: Vec<f64>, unit: &str| {
        Dataset::new(
            TimeAxis::new(vec![0.0, 1.0]).unwrap(),
            [NumericColumn::new(Identifier::new("x").unwrap(), values).with_unit(unit)],
        )
        .unwrap()
    };
    let baseline = make(vec![1.0, 2.0], "m");
    assert_eq!(
        baseline.content_fingerprint(),
        baseline.content_fingerprint()
    );
    assert_ne!(
        baseline.content_fingerprint(),
        make(vec![1.0, 3.0], "m").content_fingerprint()
    );
    assert_ne!(
        baseline.content_fingerprint(),
        make(vec![1.0, 2.0], "s").content_fingerprint()
    );
}
