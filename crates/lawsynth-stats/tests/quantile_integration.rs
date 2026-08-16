use lawsynth_stats::{StatsError, median, quantile, quantile_sorted};

#[test]
fn quantiles_are_linear_and_independent_of_input_order() {
    assert_eq!(median(&[3.0, 1.0, 5.0, 2.0]).unwrap(), 2.5);
    assert_eq!(quantile(&[3.0, 1.0, 5.0, 2.0], 0.25).unwrap(), 1.75);
    assert_eq!(quantile_sorted(&[1.0, 2.0, 3.0], 0.5), 2.0);
    assert_eq!(quantile(&[1.0], 2.0), Err(StatsError::InvalidProbability));
}
