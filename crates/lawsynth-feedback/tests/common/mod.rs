//! Shared helpers for the feedback integration tests.
//!
//! Each integration binary pulls in the whole module but uses a different
//! subset, so unused-helper warnings are expected and silenced here.
#![allow(dead_code)]

use lawsynth_feedback::{Complex, Matrix};

/// Builds a matrix from row-major data, panicking on malformed input.
pub fn matrix(rows: Vec<Vec<f64>>) -> Matrix {
    Matrix::from_rows(rows).expect("valid matrix")
}

/// Sorts complex numbers into a canonical (re, then im) order for comparison.
pub fn sorted(mut values: Vec<Complex>) -> Vec<Complex> {
    values.sort_by(|a, b| a.re.total_cmp(&b.re).then_with(|| a.im.total_cmp(&b.im)));
    values
}

/// Asserts two pole multisets match to `tolerance`, order-independent.
///
/// Each expected pole is matched greedily to the nearest unused actual pole,
/// which is robust to tiny numerical reordering of near-equal poles (where a
/// sort-then-zip would break on the tie-break coordinate).
pub fn assert_poles_match(actual: &[Complex], expected: &[Complex], tolerance: f64) {
    assert_eq!(actual.len(), expected.len(), "pole count differs");
    let mut used = vec![false; actual.len()];
    for want in expected {
        let mut best: Option<(usize, f64)> = None;
        for (index, got) in actual.iter().enumerate() {
            if used[index] {
                continue;
            }
            let distance = (got.re - want.re).hypot(got.im - want.im);
            if best.is_none_or(|(_, d)| distance < d) {
                best = Some((index, distance));
            }
        }
        let (index, distance) = best.expect("an unused actual pole");
        assert!(
            distance < tolerance,
            "pole mismatch: want {want}, nearest {} at distance {distance}",
            actual[index]
        );
        used[index] = true;
    }
}

/// The largest absolute entry of `a − b` for equally shaped matrices.
pub fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            best = best.max((a.get(i, j) - b.get(i, j)).abs());
        }
    }
    best
}
