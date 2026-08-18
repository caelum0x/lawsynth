//! Typed-error boundary tests for the discrete designs.

mod common;

use common::matrix;
use lawsynth_discrete::{DiscreteError, Matrix, discrete_kalman, dlqr};

#[test]
fn dlqr_rejects_indefinite_r() {
    let a = matrix(vec![vec![1.0, 0.1], vec![0.0, 1.0]]);
    let b = matrix(vec![vec![0.0], vec![0.1]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![-1.0]]);
    assert_eq!(dlqr(&a, &b, &q, &r).unwrap_err(), DiscreteError::NotPositiveDefinite);
}

#[test]
fn dlqr_rejects_non_symmetric_r() {
    let a = matrix(vec![vec![1.0, 0.1], vec![0.0, 1.0]]);
    let b = matrix(vec![vec![0.0, 0.0], vec![0.1, 0.1]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0, 2.0], vec![0.0, 1.0]]);
    assert_eq!(dlqr(&a, &b, &q, &r).unwrap_err(), DiscreteError::NotSymmetric);
}

#[test]
fn dlqr_rejects_negative_definite_q() {
    let a = matrix(vec![vec![1.0, 0.1], vec![0.0, 1.0]]);
    let b = matrix(vec![vec![0.0], vec![0.1]]);
    let q = matrix(vec![vec![-1.0, 0.0], vec![0.0, -1.0]]);
    let r = matrix(vec![vec![1.0]]);
    assert_eq!(dlqr(&a, &b, &q, &r).unwrap_err(), DiscreteError::NotPositiveSemidefinite);
}

#[test]
fn dlqr_rejects_non_square_a() {
    let a = matrix(vec![vec![1.0, 0.1, 0.0], vec![0.0, 1.0, 0.0]]);
    let b = matrix(vec![vec![0.0], vec![0.1]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0]]);
    assert_eq!(dlqr(&a, &b, &q, &r).unwrap_err(), DiscreteError::NonSquare);
}

#[test]
fn dlqr_rejects_shape_mismatch() {
    let a = matrix(vec![vec![1.0, 0.1], vec![0.0, 1.0]]);
    let b = matrix(vec![vec![0.0], vec![0.1], vec![0.0]]); // 3 rows, A is order 2
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0]]);
    assert_eq!(dlqr(&a, &b, &q, &r).unwrap_err(), DiscreteError::ShapeMismatch);
}

#[test]
fn dlqr_reports_non_convergence_for_uncontrollable_unstable_mode() {
    // State 1 is unstable (1.5) and entirely unactuated (B row is zero) and
    // decoupled — no gain can stabilize it, so the DARE iterate diverges.
    let a = matrix(vec![vec![0.5, 0.0], vec![0.0, 1.5]]);
    let b = matrix(vec![vec![1.0], vec![0.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0]]);
    assert_eq!(dlqr(&a, &b, &q, &r).unwrap_err(), DiscreteError::NotConvergent);
}

#[test]
fn kalman_rejects_indefinite_r() {
    let a = matrix(vec![vec![0.9, 0.1], vec![0.0, 0.8]]);
    let c = matrix(vec![vec![1.0, 0.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![-0.5]]);
    assert_eq!(discrete_kalman(&a, &c, &q, &r).unwrap_err(), DiscreteError::NotPositiveDefinite);
}

#[test]
fn kalman_rejects_shape_mismatch() {
    let a = matrix(vec![vec![0.9, 0.1], vec![0.0, 0.8]]);
    let c = matrix(vec![vec![1.0, 0.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![0.1, 0.0], vec![0.0, 0.1]]); // 2×2 but one output
    assert_eq!(discrete_kalman(&a, &c, &q, &r).unwrap_err(), DiscreteError::ShapeMismatch);
}

#[test]
fn kalman_reports_non_convergence_for_undetectable_unstable_mode() {
    // State 1 is unstable (1.5) and unmeasured (C sees only state 0) and
    // decoupled — undetectable, so the dual DARE iterate diverges.
    let a = matrix(vec![vec![0.5, 0.0], vec![0.0, 1.5]]);
    let c = matrix(vec![vec![1.0, 0.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![0.1]]);
    assert_eq!(discrete_kalman(&a, &c, &q, &r).unwrap_err(), DiscreteError::NotConvergent);
}
