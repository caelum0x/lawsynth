//! Integration tests for infinite-horizon LQR (Kleinman/Riccati iteration).

mod common;

use common::matrix;
use lawsynth_feedback::{FeedbackError, Matrix, lqr};

/// The double integrator with unit weights `Q = I`, `R = [1]`.
fn double_integrator_problem() -> (Matrix, Matrix, Matrix, Matrix) {
    let a = matrix(vec![vec![0.0, 1.0], vec![0.0, 0.0]]);
    let b = matrix(vec![vec![0.0], vec![1.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0]]);
    (a, b, q, r)
}

/// The CARE residual `AᵀP + PA − PBR⁻¹BᵀP + Q` as a max-norm.
fn care_residual(a: &Matrix, b: &Matrix, q: &Matrix, r: &Matrix, p: &Matrix) -> f64 {
    let n = a.rows();
    // AᵀP + PA
    let atp = a.transpose().matmul(p).unwrap();
    let pa = p.matmul(a).unwrap();
    // P B R⁻¹ Bᵀ P
    let r_inv = matrix(invert(r));
    let pb = p.matmul(b).unwrap();
    let pbr = pb.matmul(&r_inv).unwrap();
    let pbrbtp = pbr.matmul(&b.transpose().matmul(p).unwrap()).unwrap();

    let mut worst = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            let value = atp.get(i, j) + pa.get(i, j) - pbrbtp.get(i, j) + q.get(i, j);
            worst = worst.max(value.abs());
        }
    }
    worst
}

/// A tiny local inverse for the residual check (mirrors the crate's solver).
#[allow(clippy::needless_range_loop)]
fn invert(a: &Matrix) -> Vec<Vec<f64>> {
    let n = a.rows();
    let mut work: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut row = vec![0.0; 2 * n];
            for j in 0..n {
                row[j] = a.get(i, j);
            }
            row[n + i] = 1.0;
            row
        })
        .collect();
    for col in 0..n {
        let diagonal = work[col][col];
        for j in 0..2 * n {
            work[col][j] /= diagonal;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = work[row][col];
            for j in 0..2 * n {
                work[row][j] -= factor * work[col][j];
            }
        }
    }
    (0..n).map(|i| (0..n).map(|j| work[i][n + j]).collect()).collect()
}

#[test]
fn double_integrator_gain_matches_the_analytic_solution() {
    // Classic result: K = [1, √3], P = [[√3, 1], [1, √3]].
    let (a, b, q, r) = double_integrator_problem();
    let gain = lqr(&a, &b, &q, &r).unwrap();
    assert!((gain.k.get(0, 0) - 1.0).abs() < 1e-8);
    assert!((gain.k.get(0, 1) - 3.0_f64.sqrt()).abs() < 1e-8);
}

#[test]
fn double_integrator_riccati_matrix_matches_analytic() {
    let (a, b, q, r) = double_integrator_problem();
    let gain = lqr(&a, &b, &q, &r).unwrap();
    let p = gain.p.expect("LQR returns P");
    let root3 = 3.0_f64.sqrt();
    assert!((p.get(0, 0) - root3).abs() < 1e-8);
    assert!((p.get(0, 1) - 1.0).abs() < 1e-8);
    assert!((p.get(1, 0) - 1.0).abs() < 1e-8);
    assert!((p.get(1, 1) - root3).abs() < 1e-8);
}

#[test]
fn closed_loop_is_stable() {
    let (a, b, q, r) = double_integrator_problem();
    let gain = lqr(&a, &b, &q, &r).unwrap();
    assert!(gain.is_stable(1e-9));
    assert!(gain.achieved_poles.iter().all(|pole| pole.re < 0.0));
}

#[test]
fn riccati_matrix_is_symmetric() {
    let (a, b, q, r) = double_integrator_problem();
    let gain = lqr(&a, &b, &q, &r).unwrap();
    let p = gain.p.unwrap();
    assert!((p.get(0, 1) - p.get(1, 0)).abs() < 1e-14);
}

#[test]
fn care_residual_is_near_zero() {
    let (a, b, q, r) = double_integrator_problem();
    let gain = lqr(&a, &b, &q, &r).unwrap();
    let residual = care_residual(&a, &b, &q, &r, gain.p.as_ref().unwrap());
    assert!(residual < 1e-8, "CARE residual too large: {residual}");
}

#[test]
fn solves_a_stable_but_coupled_3x3_system() {
    let a = matrix(vec![vec![-1.0, 2.0, 0.0], vec![0.0, -1.0, 1.0], vec![1.0, 0.0, -2.0]]);
    let b = matrix(vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]]);
    let q = Matrix::identity(3);
    let r = Matrix::identity(2);
    let gain = lqr(&a, &b, &q, &r).unwrap();
    assert!(gain.is_stable(1e-9));
    let residual = care_residual(&a, &b, &q, &r, gain.p.as_ref().unwrap());
    assert!(residual < 1e-7, "CARE residual too large: {residual}");
}

#[test]
fn multi_input_lqr_produces_full_gain() {
    let a = matrix(vec![vec![0.0, 1.0], vec![-1.0, 0.0]]);
    let b = matrix(vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
    let q = Matrix::identity(2);
    let r = Matrix::identity(2);
    let gain = lqr(&a, &b, &q, &r).unwrap();
    assert_eq!((gain.k.rows(), gain.k.cols()), (2, 2));
    assert!(gain.is_stable(1e-9));
}

#[test]
fn non_symmetric_r_is_rejected() {
    let a = matrix(vec![vec![0.0, 1.0], vec![0.0, 0.0]]);
    let b = matrix(vec![vec![0.0, 0.0], vec![1.0, 1.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0, 2.0], vec![0.0, 1.0]]);
    assert_eq!(lqr(&a, &b, &q, &r).unwrap_err(), FeedbackError::NotSymmetric);
}

#[test]
fn indefinite_r_is_rejected() {
    let (a, b, q, _) = double_integrator_problem();
    let r = matrix(vec![vec![-1.0]]);
    assert_eq!(lqr(&a, &b, &q, &r).unwrap_err(), FeedbackError::NotPositiveDefinite);
}

#[test]
fn negative_definite_q_is_rejected() {
    let (a, b, _, r) = double_integrator_problem();
    let q = matrix(vec![vec![-1.0, 0.0], vec![0.0, -1.0]]);
    assert_eq!(lqr(&a, &b, &q, &r).unwrap_err(), FeedbackError::NotPositiveSemidefinite);
}

#[test]
fn non_symmetric_q_is_rejected() {
    let (a, b, _, r) = double_integrator_problem();
    let q = matrix(vec![vec![1.0, 2.0], vec![0.0, 1.0]]);
    assert_eq!(lqr(&a, &b, &q, &r).unwrap_err(), FeedbackError::NotSymmetric);
}
