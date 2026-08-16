use lawsynth_sparse::{NonnegativeConfig, RegressionProblem, nonnegative_least_squares};

fn main() {
    let problem = RegressionProblem::new(
        vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]],
        vec![2.0, 3.0, 5.0],
    )
    .unwrap();
    let solution = nonnegative_least_squares(&problem, &NonnegativeConfig::default()).unwrap();
    println!(
        "nonnegative coefficients: {:?}, RSS={}",
        solution.coefficients, solution.residual_sum_squares
    );
}
