//! Integration tests for observer and Kalman-filter design.

mod common;

use common::{
    bits_identical, damped_oscillator, diag2, double_integrator, filter_care_residual,
    is_positive_definite, is_symmetric, poles_match, real_poles, scalar,
};

use lawsynth_estimate::{
    Complex, EstimateError, Matrix, ObserverMethod, design_observer, is_observable, kalman_filter,
    observability_matrix,
};

const TOL: f64 = 1e-9;

#[test]
fn observer_places_error_poles_at_targets() {
    let (a, _b, c) = double_integrator();
    let observer = design_observer(&a, &c, &real_poles(&[-2.0, -3.0])).unwrap();
    assert_eq!(observer.method, ObserverMethod::PolePlacement);
    // Error dynamics A − L C must have exactly the requested spectrum.
    assert!(
        poles_match(&observer.error_poles, &[-2.0, -3.0], 1e-7),
        "achieved poles {:?}",
        observer.error_poles
    );
    assert!(observer.is_convergent(1e-6));
}

#[test]
fn observer_gain_matches_hand_computation() {
    // For A=[[0,1],[0,0]], C=[1,0], placing {−2,−3} gives A−LC=[[−L0,1],[−L1,0]]
    // with characteristic polynomial s² + L0 s + L1 = s² + 5 s + 6 ⇒ L=[5,6].
    let (a, _b, c) = double_integrator();
    let observer = design_observer(&a, &c, &real_poles(&[-2.0, -3.0])).unwrap();
    assert!((observer.gain.get(0, 0) - 5.0).abs() < 1e-7);
    assert!((observer.gain.get(1, 0) - 6.0).abs() < 1e-7);
    assert_eq!(observer.gain.rows(), 2);
    assert_eq!(observer.gain.cols(), 1);
    assert!(observer.covariance.is_none());
}

#[test]
fn observability_matrix_and_predicate_agree() {
    let (a, _b, c) = double_integrator();
    let o = observability_matrix(&a, &c).unwrap();
    assert_eq!((o.rows(), o.cols()), (2, 2));
    assert!(is_observable(&a, &c).unwrap());

    // Measuring velocity only (C=[0,1]) cannot observe position: O is rank 1.
    let c_vel = Matrix::from_rows(vec![vec![0.0, 1.0]]).unwrap();
    assert!(!is_observable(&a, &c_vel).unwrap());
}

#[test]
fn unobservable_system_is_rejected() {
    let (a, _b, _c) = double_integrator();
    let c_vel = Matrix::from_rows(vec![vec![0.0, 1.0]]).unwrap();
    let error = design_observer(&a, &c_vel, &real_poles(&[-2.0, -3.0])).unwrap_err();
    assert_eq!(error, EstimateError::Unobservable);
}

#[test]
fn multi_output_placement_is_rejected() {
    let (a, _b, _c) = double_integrator();
    let c_full = Matrix::identity(2); // p = 2 outputs
    let error = design_observer(&a, &c_full, &real_poles(&[-2.0, -3.0])).unwrap_err();
    assert_eq!(error, EstimateError::MultiOutput);
}

#[test]
fn pole_count_mismatch_is_rejected() {
    let (a, _b, c) = double_integrator();
    let error = design_observer(&a, &c, &real_poles(&[-2.0])).unwrap_err();
    assert_eq!(error, EstimateError::PoleCountMismatch);
}

#[test]
fn non_conjugate_poles_are_rejected() {
    let (a, _b, c) = double_integrator();
    // One complex pole without its conjugate ⇒ complex gain ⇒ rejected.
    let poles = vec![Complex::new(-1.0, 1.0), Complex::real(-2.0)];
    let error = design_observer(&a, &c, &poles).unwrap_err();
    assert_eq!(error, EstimateError::NonRealDesignPoles);
}

#[test]
fn dimension_mismatch_is_rejected() {
    let (a, _b, _c) = double_integrator();
    let c_wide = Matrix::from_rows(vec![vec![1.0, 0.0, 0.0]]).unwrap(); // 1×3, cols ≠ n
    let error = design_observer(&a, &c_wide, &real_poles(&[-2.0, -3.0])).unwrap_err();
    assert_eq!(error, EstimateError::ShapeMismatch);
}

#[test]
fn conjugate_complex_poles_are_accepted() {
    let (a, _b, c) = double_integrator();
    let poles = vec![Complex::new(-1.0, 2.0), Complex::new(-1.0, -2.0)];
    let observer = design_observer(&a, &c, &poles).unwrap();
    // Achieved spectrum should be the conjugate pair −1 ± 2i.
    assert!(observer.error_poles.iter().all(|p| (p.re + 1.0).abs() < 1e-7));
    assert!(observer.error_poles.iter().any(|p| (p.im.abs() - 2.0).abs() < 1e-7));
    assert!(observer.is_convergent(1e-6));
}

#[test]
fn kalman_scalar_matches_analytic_gain() {
    // Scalar filter CARE: 2aP − P²/r + q = 0 ⇒ P = ar + sqrt(a²r² + qr).
    // For a=−1, q=1, r=1: P = −1 + √2, L = P/r = −1 + √2.
    let a = scalar(-1.0);
    let c = scalar(1.0);
    let q = scalar(1.0);
    let r = scalar(1.0);
    let observer = kalman_filter(&a, &c, &q, &r).unwrap();
    let expected = -1.0 + 2.0_f64.sqrt();
    assert!((observer.gain.get(0, 0) - expected).abs() < TOL);
    let p = observer.covariance.as_ref().unwrap();
    assert!((p.get(0, 0) - expected).abs() < TOL);
    assert_eq!(observer.method, ObserverMethod::Kalman);
}

#[test]
fn kalman_covariance_is_symmetric_positive_definite() {
    let (a, _b, c) = damped_oscillator();
    let q = diag2(1.0, 1.0);
    let r = scalar(0.5);
    let observer = kalman_filter(&a, &c, &q, &r).unwrap();
    let p = observer.covariance.as_ref().unwrap();
    assert!(is_symmetric(p, 1e-10));
    assert!(is_positive_definite(p));
}

#[test]
fn kalman_filter_care_residual_is_near_zero() {
    // The headline correctness check: plug P back into the filter CARE.
    let (a, _b, c) = damped_oscillator();
    let q = diag2(2.0, 3.0);
    let r = scalar(0.5);
    let observer = kalman_filter(&a, &c, &q, &r).unwrap();
    let p = observer.covariance.as_ref().unwrap();
    let residual = filter_care_residual(&a, &c, &q, &[0.5], p);
    assert!(residual < 1e-9, "filter CARE residual {residual} too large");
}

#[test]
fn kalman_error_dynamics_are_stable() {
    let (a, _b, c) = damped_oscillator();
    let observer = kalman_filter(&a, &c, &diag2(1.0, 1.0), &scalar(0.25)).unwrap();
    assert!(observer.is_convergent(1e-9), "poles {:?}", observer.error_poles);
}

#[test]
fn kalman_supports_multiple_outputs() {
    // Full-state measurement (p = 2) where SISO Ackermann cannot apply, but the
    // dual multi-input LQR does.
    let (a, _b, _c) = damped_oscillator();
    let c_full = Matrix::identity(2);
    let q = diag2(1.0, 1.0);
    let r = diag2(0.5, 0.5);
    let observer = kalman_filter(&a, &c_full, &q, &r).unwrap();
    assert_eq!(observer.outputs(), 2);
    let p = observer.covariance.as_ref().unwrap();
    let residual = filter_care_residual(&a, &c_full, &q, &[0.5, 0.5], p);
    assert!(residual < 1e-9, "multi-output CARE residual {residual}");
}

#[test]
fn kalman_rejects_non_positive_definite_measurement_cov() {
    let (a, _b, c) = damped_oscillator();
    let error = kalman_filter(&a, &c, &diag2(1.0, 1.0), &scalar(0.0)).unwrap_err();
    assert_eq!(error, EstimateError::NotPositiveDefinite);
}

#[test]
fn kalman_rejects_non_symmetric_process_cov() {
    let (a, _b, c) = damped_oscillator();
    let q = Matrix::from_rows(vec![vec![1.0, 0.5], vec![-0.5, 1.0]]).unwrap();
    let error = kalman_filter(&a, &c, &q, &scalar(0.5)).unwrap_err();
    assert_eq!(error, EstimateError::NotSymmetric);
}

#[test]
fn kalman_rejects_indefinite_process_cov() {
    let (a, _b, c) = damped_oscillator();
    // Symmetric but indefinite (eigenvalues ±1).
    let q = Matrix::from_rows(vec![vec![0.0, 1.0], vec![1.0, 0.0]]).unwrap();
    let error = kalman_filter(&a, &c, &q, &scalar(0.5)).unwrap_err();
    assert_eq!(error, EstimateError::NotPositiveSemidefinite);
}

#[test]
fn kalman_rejects_measurement_cov_shape_mismatch() {
    let (a, _b, c) = damped_oscillator(); // p = 1, expects R 1×1
    let error = kalman_filter(&a, &c, &diag2(1.0, 1.0), &diag2(0.5, 0.5)).unwrap_err();
    assert_eq!(error, EstimateError::ShapeMismatch);
}

#[test]
fn design_is_bit_identical_across_runs() {
    let (a, _b, c) = damped_oscillator();

    let observer_a = design_observer(&a, &c, &real_poles(&[-4.0, -5.0])).unwrap();
    let observer_b = design_observer(&a, &c, &real_poles(&[-4.0, -5.0])).unwrap();
    assert!(bits_identical(&observer_a.gain, &observer_b.gain));

    let k_a = kalman_filter(&a, &c, &diag2(1.0, 2.0), &scalar(0.3)).unwrap();
    let k_b = kalman_filter(&a, &c, &diag2(1.0, 2.0), &scalar(0.3)).unwrap();
    assert!(bits_identical(&k_a.gain, &k_b.gain));
    assert!(bits_identical(k_a.covariance.as_ref().unwrap(), k_b.covariance.as_ref().unwrap()));
}
