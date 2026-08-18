//! Integration tests for the discrete Kalman filter (dual filter DARE).

mod common;

use common::{filter_dare_residual, matrix, norm, spectral_radius, step_input};
use lawsynth_discrete::{Matrix, ObserverMethod, discrete_kalman};

/// A stable plant with a single measured state (`C` selects state 0).
fn partial_output_system() -> (Matrix, Matrix, Matrix, Matrix, Matrix) {
    let a = matrix(vec![vec![0.9, 0.1], vec![0.0, 0.8]]);
    let b = matrix(vec![vec![0.0], vec![1.0]]);
    let c = matrix(vec![vec![1.0, 0.0]]);
    let q = matrix(vec![vec![0.01, 0.0], vec![0.0, 0.01]]);
    let r = matrix(vec![vec![0.1]]);
    (a, b, c, q, r)
}

#[test]
fn filter_dare_residual_is_near_zero() {
    let (a, _b, c, q, r) = partial_output_system();
    let observer = discrete_kalman(&a, &c, &q, &r).unwrap();
    let p = observer.p.expect("Kalman returns covariance");
    let residual = filter_dare_residual(&a, &c, &q, &r, &p);
    assert!(residual < 1e-9, "filter DARE residual too large: {residual}");
    assert_eq!(observer.method, ObserverMethod::Kalman);
}

#[test]
fn error_dynamics_are_inside_unit_circle() {
    let (a, _b, c, q, r) = partial_output_system();
    let observer = discrete_kalman(&a, &c, &q, &r).unwrap();
    assert!(
        observer.is_convergent(1e-9),
        "error dynamics not discrete-stable: {:?}",
        observer.error_poles
    );
    assert!(spectral_radius(&observer.error_poles) < 1.0);
    assert_eq!((observer.l.rows(), observer.l.cols()), (2, 1));
}

#[test]
fn covariance_is_symmetric_positive_definite() {
    let (a, _b, c, q, r) = partial_output_system();
    let p = discrete_kalman(&a, &c, &q, &r).unwrap().p.unwrap();
    assert!((p.get(0, 1) - p.get(1, 0)).abs() < 1e-14, "P not symmetric");
    assert!(p.get(0, 0) > 0.0);
    assert!(p.get(0, 0) * p.get(1, 1) - p.get(0, 1) * p.get(1, 0) > 0.0);
}

#[test]
fn scalar_kalman_matches_the_analytic_root() {
    // Dual of the scalar control DARE: a = 2, q = 1, r = 1 gives p = 2 + √5,
    // predictor gain L = a p / (r + p) = 2p/(1+p), error a − L inside |·| < 1.
    let a = matrix(vec![vec![2.0]]);
    let c = matrix(vec![vec![1.0]]);
    let q = matrix(vec![vec![1.0]]);
    let r = matrix(vec![vec![1.0]]);
    let observer = discrete_kalman(&a, &c, &q, &r).unwrap();

    let p_exact = 2.0 + 5.0_f64.sqrt();
    let l_exact = 2.0 * p_exact / (1.0 + p_exact);
    assert!((observer.p.as_ref().unwrap().get(0, 0) - p_exact).abs() < 1e-8);
    assert!((observer.l.get(0, 0) - l_exact).abs() < 1e-8);
    assert!(observer.error_poles[0].abs() < 1.0);
}

#[test]
fn multi_output_kalman_is_supported() {
    // Two outputs: C is 2×2 (both states measured, mixed).
    let a = matrix(vec![vec![0.95, 0.1], vec![0.0, 0.9]]);
    let c = matrix(vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![0.5, 0.0], vec![0.0, 0.5]]);
    let observer = discrete_kalman(&a, &c, &q, &r).unwrap();
    assert_eq!((observer.l.rows(), observer.l.cols()), (2, 2));
    assert!(observer.is_convergent(1e-9));
    let residual = filter_dare_residual(&a, &c, &q, &r, observer.p.as_ref().unwrap());
    assert!(residual < 1e-8, "residual = {residual}");
}

#[test]
fn observer_reconstructs_state_from_partial_measurements() {
    let (a, b, c, q, r) = partial_output_system();
    let observer = discrete_kalman(&a, &c, &q, &r).unwrap();
    let l = &observer.l;

    // Drive the plant with a constant input so its state stays nonzero, and run
    // the predictor observer x̂_{k+1} = A x̂ + B u + L (y − C x̂) from a wrong x̂0.
    let mut x = vec![1.0, -1.0];
    let mut x_hat = vec![0.0, 0.0];
    let u = vec![1.0];

    for _ in 0..200 {
        let y = step(&c, &x); // measurement y = C x
        let y_hat = step(&c, &x_hat);
        let innovation: Vec<f64> = y.iter().zip(&y_hat).map(|(a, b)| a - b).collect();
        let l_innov = step(l, &innovation);
        let x_next = step_input(&a, &b, &x, &u);
        let mut x_hat_next = step_input(&a, &b, &x_hat, &u);
        for i in 0..x_hat_next.len() {
            x_hat_next[i] += l_innov[i];
        }
        x = x_next;
        x_hat = x_hat_next;
    }

    let error: Vec<f64> = x.iter().zip(&x_hat).map(|(a, b)| a - b).collect();
    assert!(norm(&error) < 1e-6, "estimate did not converge: ‖x − x̂‖ = {}", norm(&error));
}

/// `A x` for a general matrix and vector (local, avoids importing more helpers).
fn step(a: &Matrix, x: &[f64]) -> Vec<f64> {
    (0..a.rows()).map(|i| (0..a.cols()).map(|j| a.get(i, j) * x[j]).sum()).collect()
}
