use lawsynth_stats::{StatsError, covariance, pearson_correlation};

#[test]
fn covariance_and_correlation_are_symmetric_for_aligned_data() {
    let left = [1.0, 2.0, 3.0];
    let right = [2.0, 4.0, 8.0];
    assert_eq!(covariance(&left, &right), covariance(&right, &left));
    assert_eq!(pearson_correlation(&left, &left).unwrap(), 1.0);
    assert_eq!(pearson_correlation(&left, &[1.0, 2.0]), Err(StatsError::LengthMismatch));
}
