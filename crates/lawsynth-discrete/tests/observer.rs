//! Integration tests for the discrete Luenberger observer (dual pole placement).

mod common;

use common::{assert_poles_match, matrix, norm, spectral_radius, step_input};
use lawsynth_discrete::{
    Complex, DiscreteError, Matrix, ObserverMethod, discrete_observer_from_poles,
};

/// A stable, observable single-output plant.
fn plant() -> (Matrix, Matrix, Matrix) {
    let a = matrix(vec![vec![0.9, 0.1], vec![0.0, 0.8]]);
    let b = matrix(vec![vec![0.0], vec![1.0]]);
    let c = matrix(vec![vec![1.0, 0.0]]);
    (a, b, c)
}

#[test]
fn places_error_poles_in_the_z_plane() {
    let (a, _b, c) = plant();
    let desired = [Complex::real(0.2), Complex::real(0.3)];
    let observer = discrete_observer_from_poles(&a, &c, &desired).unwrap();
    assert_eq!(observer.method, ObserverMethod::PolePlacement);
    assert!(observer.p.is_none());
    assert_poles_match(&observer.error_poles, &desired, 1e-9);
    assert!(observer.is_convergent(1e-6));
    assert!(spectral_radius(&observer.error_poles) < 1.0);
}

#[test]
fn places_complex_conjugate_error_poles() {
    let (a, _b, c) = plant();
    let desired = [Complex::new(0.3, 0.2), Complex::new(0.3, -0.2)];
    let observer = discrete_observer_from_poles(&a, &c, &desired).unwrap();
    assert_poles_match(&observer.error_poles, &desired, 1e-8);
    assert_eq!((observer.l.rows(), observer.l.cols()), (2, 1));
}

#[test]
fn placement_observer_reconstructs_state() {
    let (a, b, c) = plant();
    let desired = [Complex::real(0.2), Complex::real(0.25)];
    let observer = discrete_observer_from_poles(&a, &c, &desired).unwrap();
    let l = &observer.l;

    let mut x = vec![1.0, -1.0];
    let mut x_hat = vec![0.0, 0.0];
    let u = vec![1.0];
    for _ in 0..200 {
        let y = mat_vec(&c, &x);
        let y_hat = mat_vec(&c, &x_hat);
        let innovation: Vec<f64> = y.iter().zip(&y_hat).map(|(p, q)| p - q).collect();
        let l_innov = mat_vec(l, &innovation);
        let x_next = step_input(&a, &b, &x, &u);
        let mut x_hat_next = step_input(&a, &b, &x_hat, &u);
        for i in 0..x_hat_next.len() {
            x_hat_next[i] += l_innov[i];
        }
        x = x_next;
        x_hat = x_hat_next;
    }
    let error: Vec<f64> = x.iter().zip(&x_hat).map(|(p, q)| p - q).collect();
    assert!(norm(&error) < 1e-6, "estimate did not converge: ‖x − x̂‖ = {}", norm(&error));
}

#[test]
fn rejects_multi_output_placement() {
    let a = matrix(vec![vec![0.9, 0.1], vec![0.0, 0.8]]);
    let c = matrix(vec![vec![1.0, 0.0], vec![0.0, 1.0]]); // 2×2, two outputs
    let desired = [Complex::real(0.2), Complex::real(0.3)];
    assert_eq!(
        discrete_observer_from_poles(&a, &c, &desired).unwrap_err(),
        DiscreteError::MultiOutput
    );
}

#[test]
fn rejects_unobservable_pair() {
    // C measures only state 0; with a diagonal A the second state is invisible.
    let a = matrix(vec![vec![0.9, 0.0], vec![0.0, 0.8]]);
    let c = matrix(vec![vec![1.0, 0.0]]);
    let desired = [Complex::real(0.2), Complex::real(0.3)];
    assert_eq!(
        discrete_observer_from_poles(&a, &c, &desired).unwrap_err(),
        DiscreteError::Unobservable
    );
}

#[test]
fn rejects_lone_complex_pole() {
    let (a, _b, c) = plant();
    let desired = [Complex::new(0.3, 0.2), Complex::real(0.4)]; // not conjugate-closed
    assert_eq!(
        discrete_observer_from_poles(&a, &c, &desired).unwrap_err(),
        DiscreteError::NonRealDesignPoles
    );
}

#[test]
fn rejects_wrong_pole_count() {
    let (a, _b, c) = plant();
    let desired = [Complex::real(0.2)]; // only one pole for a 2-state system
    assert_eq!(
        discrete_observer_from_poles(&a, &c, &desired).unwrap_err(),
        DiscreteError::PoleCountMismatch
    );
}

/// `A x` for a general matrix and vector.
fn mat_vec(a: &Matrix, x: &[f64]) -> Vec<f64> {
    (0..a.rows()).map(|i| (0..a.cols()).map(|j| a.get(i, j) * x[j]).sum()).collect()
}
