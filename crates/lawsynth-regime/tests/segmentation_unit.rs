use lawsynth_regime::{SegmentationConfig, pelt};
#[test]
fn pelt_finds_clear_level_change() {
    let values = [0.0, 0.1, -0.1, 0.0, 0.05, 5.0, 5.1, 4.9, 5.0, 5.05];
    let result = pelt(
        &values,
        SegmentationConfig {
            penalty: 1.0,
            min_segment_len: 3,
        },
    )
    .unwrap();
    assert_eq!(result.change_points(), vec![5]);
    assert_eq!(result.segments.len(), 2);
}
