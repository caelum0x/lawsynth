use lawsynth_score::{ResidualSummary, fit_statistics, residuals};

#[test]
fn fit_and_residual_diagnostics_are_consistent_for_one_prediction_series() {
    let observed = [1.0, 3.0, 5.0, 7.0];
    let predicted = [1.0, 2.0, 6.0, 7.0];
    let residual = residuals(&observed, &predicted).unwrap();
    let summary = ResidualSummary::from_residuals(&residual).unwrap();
    let fit = fit_statistics(&observed, &predicted).unwrap();
    assert_eq!(residual, vec![0.0, 1.0, -1.0, 0.0]);
    assert_eq!(summary.maximum_absolute, 1.0);
    assert_eq!(fit.residual_sum_squares, 2.0);
    assert_eq!(fit.mean_squared_error, 0.5);
}
