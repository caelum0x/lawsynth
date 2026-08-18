//! Integration tests for single-input pole placement (Ackermann).

mod common;

use common::{assert_poles_match, matrix};
use lawsynth_feedback::{Complex, FeedbackError, place_poles};

/// The double integrator `A = [[0,1],[0,0]]`, `b = [[0],[1]]`.
fn double_integrator() -> (lawsynth_feedback::Matrix, lawsynth_feedback::Matrix) {
    (matrix(vec![vec![0.0, 1.0], vec![0.0, 0.0]]), matrix(vec![vec![0.0], vec![1.0]]))
}

#[test]
fn places_real_poles_on_double_integrator() {
    let (a, b) = double_integrator();
    let desired = [Complex::real(-1.0), Complex::real(-2.0)];
    let gain = place_poles(&a, &b, &desired).unwrap();
    assert_poles_match(&gain.achieved_poles, &desired, 1e-10);
}

#[test]
fn analytic_gain_for_double_integrator_is_exact() {
    // For A=[[0,1],[0,0]], b=[[0],[1]], poles {−1,−2}: p(s)=s²+3s+2 ⇒ K=[2,3].
    let (a, b) = double_integrator();
    let desired = [Complex::real(-1.0), Complex::real(-2.0)];
    let gain = place_poles(&a, &b, &desired).unwrap();
    assert!((gain.k.get(0, 0) - 2.0).abs() < 1e-12);
    assert!((gain.k.get(0, 1) - 3.0).abs() < 1e-12);
    assert!(gain.p.is_none(), "pole placement forms no value function");
}

#[test]
fn places_a_complex_conjugate_pair() {
    let (a, b) = double_integrator();
    let desired = [Complex::new(-1.0, 1.0), Complex::new(-1.0, -1.0)];
    let gain = place_poles(&a, &b, &desired).unwrap();
    assert_poles_match(&gain.achieved_poles, &desired, 1e-10);
    // Gain must be real: K = [c₀, c₁] = [2, 2] for (s+1)²+1 = s²+2s+2.
    assert!((gain.k.get(0, 0) - 2.0).abs() < 1e-12);
    assert!((gain.k.get(0, 1) - 2.0).abs() < 1e-12);
}

#[test]
fn gain_is_the_single_output_row() {
    let (a, b) = double_integrator();
    let desired = [Complex::real(-3.0), Complex::real(-4.0)];
    let gain = place_poles(&a, &b, &desired).unwrap();
    assert_eq!((gain.k.rows(), gain.k.cols()), (1, 2));
}

#[test]
fn places_poles_on_a_3x3_triple_integrator() {
    let a = matrix(vec![vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0], vec![0.0, 0.0, 0.0]]);
    let b = matrix(vec![vec![0.0], vec![0.0], vec![1.0]]);
    let desired = [Complex::real(-1.0), Complex::real(-2.0), Complex::real(-3.0)];
    let gain = place_poles(&a, &b, &desired).unwrap();
    assert_poles_match(&gain.achieved_poles, &desired, 1e-8);
}

#[test]
fn places_poles_on_a_3x3_with_a_complex_pair() {
    let a = matrix(vec![vec![0.0, 1.0, 0.0], vec![0.0, 0.0, 1.0], vec![0.0, 0.0, 0.0]]);
    let b = matrix(vec![vec![0.0], vec![0.0], vec![1.0]]);
    let desired = [Complex::real(-2.0), Complex::new(-1.0, 3.0), Complex::new(-1.0, -3.0)];
    let gain = place_poles(&a, &b, &desired).unwrap();
    assert_poles_match(&gain.achieved_poles, &desired, 1e-8);
}

#[test]
fn uncontrollable_system_is_rejected() {
    // Diagonal A with b aligned to one mode ⇒ controllability matrix singular.
    let a = matrix(vec![vec![1.0, 0.0], vec![0.0, 2.0]]);
    let b = matrix(vec![vec![1.0], vec![0.0]]);
    let desired = [Complex::real(-1.0), Complex::real(-2.0)];
    assert_eq!(place_poles(&a, &b, &desired).unwrap_err(), FeedbackError::Uncontrollable);
}

#[test]
fn non_conjugate_poles_are_rejected() {
    let (a, b) = double_integrator();
    // A lone complex pole (no conjugate) would give a complex gain.
    let desired = [Complex::new(-1.0, 1.0), Complex::real(-2.0)];
    assert_eq!(place_poles(&a, &b, &desired).unwrap_err(), FeedbackError::NonRealDesignPoles);
}

#[test]
fn wrong_pole_count_is_rejected() {
    let (a, b) = double_integrator();
    let desired = [Complex::real(-1.0)];
    assert_eq!(place_poles(&a, &b, &desired).unwrap_err(), FeedbackError::PoleCountMismatch);
}

#[test]
fn multi_input_b_is_rejected() {
    let a = matrix(vec![vec![0.0, 1.0], vec![0.0, 0.0]]);
    let b = matrix(vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
    let desired = [Complex::real(-1.0), Complex::real(-2.0)];
    assert_eq!(place_poles(&a, &b, &desired).unwrap_err(), FeedbackError::MultiInput);
}

#[test]
fn mismatched_b_rows_are_rejected() {
    let a = matrix(vec![vec![0.0, 1.0], vec![0.0, 0.0]]);
    let b = matrix(vec![vec![0.0], vec![1.0], vec![0.0]]);
    let desired = [Complex::real(-1.0), Complex::real(-2.0)];
    assert_eq!(place_poles(&a, &b, &desired).unwrap_err(), FeedbackError::ShapeMismatch);
}
