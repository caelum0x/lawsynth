use lawsynth_data::TimeAxis;
use lawsynth_preprocess::{PreprocessError, align_series_linear};

#[test]
fn alignment_interpolates_only_inside_source_coverage() {
    let target = TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap();
    assert_eq!(
        align_series_linear(&[0.0, 2.0], &[0.0, 4.0], &target).unwrap(),
        vec![0.0, 2.0, 4.0]
    );
    assert_eq!(
        align_series_linear(&[0.0, 2.0], &[0.0, 4.0], &TimeAxis::new(vec![3.0]).unwrap()),
        Err(PreprocessError::ResampleOutOfBounds)
    );
}
