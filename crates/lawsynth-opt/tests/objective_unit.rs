use lawsynth_opt::{OptimizationError, mean_squared_error, residual_sum_squares};

#[test]
fn objectives_validate_alignment_and_report_population_error() {
    assert_eq!(residual_sum_squares(&[1.0, 3.0], &[2.0, 1.0]).unwrap(), 5.0);
    assert_eq!(mean_squared_error(&[1.0, 3.0], &[2.0, 1.0]).unwrap(), 2.5);
    assert_eq!(
        mean_squared_error(&[1.0], &[1.0, 2.0]),
        Err(OptimizationError::LengthMismatch)
    );
}
