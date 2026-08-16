use lawsynth_sparse::{RegressionProblem, SparseConfig, sr3};

#[test]
fn sr3_is_deterministic_and_eliminates_subthreshold_features() {
    let problem = RegressionProblem::new(
        vec![
            vec![1.0, 0.0],
            vec![1.0, 1.0],
            vec![1.0, 2.0],
            vec![1.0, 3.0],
        ],
        vec![0.0, 2.0, 4.0, 6.0],
    )
    .unwrap();
    let config = SparseConfig {
        threshold: 0.1,
        ridge: 1e-4,
        max_iterations: 100,
    };
    let first = sr3(&problem, &config).unwrap();
    assert_eq!(first, sr3(&problem, &config).unwrap());
    assert!(first.coefficients[0].abs() < 0.1);
    assert!((first.coefficients[1] - 2.0).abs() < 1e-3);
}
