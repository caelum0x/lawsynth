use lawsynth_causal::{CausalError, validate_time_order};
#[test]
fn strictly_increasing_series_is_required() {
    let order = validate_time_order(&[0.0, 0.2, 1.5]).unwrap();
    assert_eq!(order.observations, 3);
    assert!(matches!(
        validate_time_order(&[0.0, 0.0]),
        Err(CausalError::NonMonotonicTime { index: 1 })
    ));
}
