use lawsynth_data::TimeAxis;

#[test]
fn time_axis_preserves_irregular_increasing_samples_and_detects_regular_grids() {
    let regular = TimeAxis::new(vec![0.0, 0.5, 1.0]).unwrap();
    let irregular = TimeAxis::new(vec![0.0, 0.5, 1.2]).unwrap();
    assert!(regular.is_regular(1e-12));
    assert!(!irregular.is_regular(1e-12));
    assert_eq!(irregular.values(), &[0.0, 0.5, 1.2]);
}
