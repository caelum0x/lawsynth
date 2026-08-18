use lawsynth_sparse::{RegressionProblem, SparseConfig, stlsq};

#[test]
fn stlsq_recovers_scale_invariant_single_feature_linear_laws() {
    for scale in [0.25, 1.0, 17.0] {
        let problem = RegressionProblem::new(
            (0..6).map(|value| vec![scale * value as f64]).collect(),
            (0..6).map(|value| 3.0 * scale * value as f64).collect(),
        )
        .unwrap();
        let solution =
            stlsq(&problem, &SparseConfig { threshold: 0.01, ..Default::default() }).unwrap();
        assert!((solution.coefficients[0] - 3.0).abs() < 1e-8);
        assert!(solution.residual_sum_squares < 1e-14);
    }
}
