//! Bit-for-bit determinism of both design methods.

mod common;

use common::matrix;
use lawsynth_feedback::{Complex, Matrix, lqr, place_poles};

/// Compares two matrices by raw `f64` bit patterns.
fn bits_equal(a: &Matrix, b: &Matrix) -> bool {
    if (a.rows(), a.cols()) != (b.rows(), b.cols()) {
        return false;
    }
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            if a.get(i, j).to_bits() != b.get(i, j).to_bits() {
                return false;
            }
        }
    }
    true
}

/// Compares two pole lists by raw `f64` bit patterns, in order.
fn poles_bits_equal(a: &[Complex], b: &[Complex]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.re.to_bits() == y.re.to_bits() && x.im.to_bits() == y.im.to_bits())
}

#[test]
fn pole_placement_is_bit_identical() {
    let a = matrix(vec![vec![0.0, 1.0], vec![0.0, 0.0]]);
    let b = matrix(vec![vec![0.0], vec![1.0]]);
    let desired = [Complex::new(-1.0, 2.0), Complex::new(-1.0, -2.0)];
    let first = place_poles(&a, &b, &desired).unwrap();
    let second = place_poles(&a, &b, &desired).unwrap();
    assert!(bits_equal(&first.k, &second.k));
    assert!(poles_bits_equal(&first.achieved_poles, &second.achieved_poles));
}

#[test]
fn lqr_is_bit_identical() {
    let a = matrix(vec![vec![0.0, 1.0], vec![0.0, 0.0]]);
    let b = matrix(vec![vec![0.0], vec![1.0]]);
    let q = Matrix::identity(2);
    let r = matrix(vec![vec![1.0]]);
    let first = lqr(&a, &b, &q, &r).unwrap();
    let second = lqr(&a, &b, &q, &r).unwrap();
    assert!(bits_equal(&first.k, &second.k));
    assert!(bits_equal(first.p.as_ref().unwrap(), second.p.as_ref().unwrap()));
    assert!(poles_bits_equal(&first.achieved_poles, &second.achieved_poles));
}
