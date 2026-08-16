use lawsynth_sparse::{RegressionProblem, SparseConfig, stlsq_standardized};

#[test]
fn standardized_fit_returns_coefficients_in_the_callers_original_feature_units() {
    let problem = RegressionProblem::new(
        vec![
            vec![1.0, 10.0],
            vec![1.0, 20.0],
            vec![1.0, 30.0],
            vec![1.0, 40.0],
        ],
        vec![4.0, 6.0, 8.0, 10.0],
    )
    .unwrap();
    let solution = stlsq_standardized(
        &problem,
        &SparseConfig {
            threshold: 1e-9,
            ..Default::default()
        },
    )
    .unwrap();
    assert!((solution.coefficients[0] - 2.0).abs() < 1e-8);
    assert!((solution.coefficients[1] - 0.2).abs() < 1e-9);
    assert!(solution.residual_sum_squares < 1e-15);
}
