use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_preprocess::{resample_linear_with_report, standardize, unstandardize};

#[test]
fn resampling_and_scale_roundtrip_preserve_interpolated_values_and_units() {
    let original = Dataset::new(
        TimeAxis::new(vec![0.0, 2.0]).unwrap(),
        [NumericColumn::new(Identifier::new("x").unwrap(), vec![1.0, 5.0]).with_unit("m")],
    )
    .unwrap();
    let (resampled, report) =
        resample_linear_with_report(&original, TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap())
            .unwrap();
    assert_eq!(resampled.columns()[&Identifier::new("x").unwrap()].values, vec![1.0, 3.0, 5.0]);
    assert_eq!(report.target_time, resampled.time().values());
    let (scaled, scale_report) = standardize(&resampled).unwrap();
    let restored = unstandardize(&scaled, &scale_report).unwrap();
    assert_eq!(restored.time(), resampled.time());
    assert_eq!(restored.columns()[&Identifier::new("x").unwrap()].unit, Some("m".into()));
    assert!(
        restored.columns()[&Identifier::new("x").unwrap()]
            .values
            .iter()
            .zip(&resampled.columns()[&Identifier::new("x").unwrap()].values)
            .all(|(actual, expected)| (actual - expected).abs() < 1e-12)
    );
}
