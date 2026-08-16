use lawsynth_sparse::{RegressionProblem, SparseError};

#[test]
fn regression_problem_validates_shape_and_finite_values() {
    assert_eq!(
        RegressionProblem::new(vec![], vec![]),
        Err(SparseError::EmptyProblem)
    );
    assert_eq!(
        RegressionProblem::new(vec![vec![1.0], vec![2.0, 3.0]], vec![1.0, 2.0]),
        Err(SparseError::RowLengthMismatch)
    );
    assert_eq!(
        RegressionProblem::new(vec![vec![f64::NAN]], vec![1.0]),
        Err(SparseError::NonFiniteValue)
    );
    assert_eq!(
        RegressionProblem::new(vec![vec![1.0, 2.0]], vec![3.0])
            .unwrap()
            .features(),
        2
    );
}
