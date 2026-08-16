use lawsynth_regime::{segment_cost, segment_moments};
#[test]
fn constant_segment_has_zero_residual_cost() {
    let values = [3.0, 3.0, 3.0];
    let moments = segment_moments(&values, 0, 3).unwrap();
    assert_eq!(moments.mean, 3.0);
    assert_eq!(segment_cost(&values, 0, 3).unwrap(), 0.0);
}
