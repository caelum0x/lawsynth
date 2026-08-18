use lawsynth_regime::{SegmentationConfig, pelt};
#[test]
fn segmentation_covers_input_without_gaps() {
    let values: Vec<f64> = (0..30).map(|i| if i < 15 { 0.0 } else { 10.0 }).collect();
    let result = pelt(&values, SegmentationConfig { penalty: 0.5, min_segment_len: 2 }).unwrap();
    assert_eq!(result.segments.first().unwrap().start, 0);
    assert_eq!(result.segments.last().unwrap().end, values.len());
    assert_eq!(result.change_points(), vec![15]);
}
