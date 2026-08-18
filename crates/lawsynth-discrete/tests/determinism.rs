//! Bit-for-bit determinism of all three discrete designs.

mod common;

use common::{bits_equal, matrix, poles_bits_equal};
use lawsynth_discrete::{Complex, Matrix, discrete_kalman, discrete_observer_from_poles, dlqr};

#[test]
fn dlqr_is_bit_identical() {
    let a = matrix(vec![vec![1.1, 0.2], vec![0.0, 1.05]]);
    let b = matrix(vec![vec![1.0], vec![1.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0]]);
    let first = dlqr(&a, &b, &q, &r).unwrap();
    let second = dlqr(&a, &b, &q, &r).unwrap();
    assert!(bits_equal(&first.k, &second.k));
    assert!(bits_equal(&first.p, &second.p));
    assert!(poles_bits_equal(&first.achieved_poles, &second.achieved_poles));
}

#[test]
fn discrete_kalman_is_bit_identical() {
    let a = matrix(vec![vec![0.9, 0.1], vec![0.0, 0.8]]);
    let c = matrix(vec![vec![1.0, 0.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![0.1]]);
    let first = discrete_kalman(&a, &c, &q, &r).unwrap();
    let second = discrete_kalman(&a, &c, &q, &r).unwrap();
    assert!(bits_equal(&first.l, &second.l));
    assert!(bits_equal(first.p.as_ref().unwrap(), second.p.as_ref().unwrap()));
    assert!(poles_bits_equal(&first.error_poles, &second.error_poles));
}

#[test]
fn observer_placement_is_bit_identical() {
    let a = matrix(vec![vec![0.9, 0.1], vec![0.0, 0.8]]);
    let c = matrix(vec![vec![1.0, 0.0]]);
    let desired = [Complex::new(0.3, 0.2), Complex::new(0.3, -0.2)];
    let first = discrete_observer_from_poles(&a, &c, &desired).unwrap();
    let second = discrete_observer_from_poles(&a, &c, &desired).unwrap();
    assert!(bits_equal(&first.l, &second.l));
    assert!(poles_bits_equal(&first.error_poles, &second.error_poles));
}
