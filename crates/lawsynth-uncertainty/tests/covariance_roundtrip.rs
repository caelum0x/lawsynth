use lawsynth_uncertainty::{CovarianceMatrix, linear_propagate};

#[test]
fn covariance_estimation_and_delta_propagation_agree() {
    let covariance = CovarianceMatrix::from_observations(&[
        vec![1.0, 2.0],
        vec![2.0, 4.0],
        vec![3.0, 6.0],
        vec![4.0, 8.0],
    ])
    .unwrap();
    assert_eq!(covariance.dimension(), 2);
    assert!((covariance.get(0, 1).unwrap() - 10.0 / 3.0).abs() < 1e-12);
    let sigma = linear_propagate(&[1.0, 0.0], &covariance).unwrap();
    assert!((sigma - (5.0 / 3.0_f64).sqrt()).abs() < 1e-12);
}
