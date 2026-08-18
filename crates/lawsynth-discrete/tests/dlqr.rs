//! Integration tests for discrete LQR (DARE value iteration).

mod common;

use common::{control_dare_residual, matrix, norm, spectral_radius, step};
use lawsynth_discrete::{Matrix, dlqr};

/// The discrete double integrator `A = [[1, dt], [0, 1]]`, `B = [[0], [dt]]`.
fn double_integrator(dt: f64) -> (Matrix, Matrix, Matrix, Matrix) {
    let a = matrix(vec![vec![1.0, dt], vec![0.0, 1.0]]);
    let b = matrix(vec![vec![0.0], vec![dt]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0]]);
    (a, b, q, r)
}

#[test]
fn double_integrator_closed_loop_is_inside_unit_circle() {
    let (a, b, q, r) = double_integrator(0.1);
    let gain = dlqr(&a, &b, &q, &r).unwrap();
    // Discrete stability: every closed-loop eigenvalue strictly inside |λ| < 1.
    assert!(gain.is_stable(1e-9), "closed loop not discrete-stable: {:?}", gain.achieved_poles);
    assert!(spectral_radius(&gain.achieved_poles) < 1.0);
}

#[test]
fn double_integrator_dare_residual_is_near_zero() {
    let (a, b, q, r) = double_integrator(0.1);
    let gain = dlqr(&a, &b, &q, &r).unwrap();
    let residual = control_dare_residual(&a, &b, &q, &r, &gain.p);
    assert!(residual < 1e-9, "DARE residual too large: {residual}");
}

#[test]
fn riccati_matrix_is_symmetric_positive_definite() {
    let (a, b, q, r) = double_integrator(0.1);
    let gain = dlqr(&a, &b, &q, &r).unwrap();
    let p = &gain.p;
    assert!((p.get(0, 1) - p.get(1, 0)).abs() < 1e-14, "P not symmetric");
    // 2×2 PD test: leading minors positive.
    assert!(p.get(0, 0) > 0.0);
    assert!(p.get(0, 0) * p.get(1, 1) - p.get(0, 1) * p.get(1, 0) > 0.0);
}

#[test]
fn scalar_dare_matches_the_analytic_root() {
    // x_{k+1} = a x + u, Q = q, R = r. The scalar DARE p² + p(r − a²r − q) − qr = 0
    // has stabilizing root p = 2 + √5 for a = 2, q = 1, r = 1, with
    // K = 2p/(1+p) and closed loop a − K = 2/(1+p) inside the unit circle.
    let a = matrix(vec![vec![2.0]]);
    let b = matrix(vec![vec![1.0]]);
    let q = matrix(vec![vec![1.0]]);
    let r = matrix(vec![vec![1.0]]);
    let gain = dlqr(&a, &b, &q, &r).unwrap();

    let p_exact = 2.0 + 5.0_f64.sqrt();
    let k_exact = 2.0 * p_exact / (1.0 + p_exact);
    assert!((gain.p.get(0, 0) - p_exact).abs() < 1e-8, "P = {}", gain.p.get(0, 0));
    assert!((gain.k.get(0, 0) - k_exact).abs() < 1e-8, "K = {}", gain.k.get(0, 0));
    assert!(gain.achieved_poles[0].abs() < 1.0);
    assert!((gain.achieved_poles[0].re - (2.0 - k_exact)).abs() < 1e-8);
}

#[test]
fn stabilizes_an_open_loop_unstable_system() {
    // Both open-loop eigenvalues (1.1, 1.05) are outside the unit circle.
    let a = matrix(vec![vec![1.1, 0.2], vec![0.0, 1.05]]);
    let b = matrix(vec![vec![1.0], vec![1.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0]]);
    let gain = dlqr(&a, &b, &q, &r).unwrap();
    assert!(gain.is_stable(1e-9), "unstable plant not stabilized: {:?}", gain.achieved_poles);

    // Simulate x_{k+1} = (A − BK) x_k from x0 and confirm ‖x_k‖ → 0.
    let bk = b.matmul(&gain.k).unwrap();
    let acl = common::sub(&a, &bk);
    let mut x = vec![1.0, -1.0];
    for _ in 0..500 {
        x = step(&acl, &x);
    }
    assert!(norm(&x) < 1e-6, "state did not decay: ‖x‖ = {}", norm(&x));
}

#[test]
fn dare_residual_near_zero_for_coupled_system() {
    let a = matrix(vec![vec![0.9, 0.3, 0.0], vec![0.0, 0.8, 0.2], vec![0.1, 0.0, 0.95]]);
    let b = matrix(vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]]);
    let q = Matrix::identity(3);
    let r = Matrix::identity(2);
    let gain = dlqr(&a, &b, &q, &r).unwrap();
    assert!(gain.is_stable(1e-9));
    let residual = control_dare_residual(&a, &b, &q, &r, &gain.p);
    assert!(residual < 1e-8, "residual = {residual}");
    assert_eq!((gain.k.rows(), gain.k.cols()), (2, 3));
}
