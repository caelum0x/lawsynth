//! Shared helpers for the discrete-time control/estimation integration tests.
//!
//! Each integration binary pulls in the whole module but uses a different
//! subset, so unused-helper warnings are expected and silenced here.
#![allow(dead_code)]

use lawsynth_discrete::{Complex, Matrix};

/// Builds a matrix from row-major data, panicking on malformed input.
pub fn matrix(rows: Vec<Vec<f64>>) -> Matrix {
    Matrix::from_rows(rows).expect("valid matrix")
}

/// A tiny independent Gauss–Jordan inverse for residual checks (mirrors the
/// crate's solver but kept separate so the test does not trust it).
#[allow(clippy::needless_range_loop)]
pub fn inverse(a: &Matrix) -> Matrix {
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
        let mut pivot = col;
        let mut best = work[col][col].abs();
        for row in col + 1..n {
            if work[row][col].abs() > best {
                best = work[row][col].abs();
                pivot = row;
            }
        }
        work.swap(col, pivot);
        let diagonal = work[col][col];
        for value in work[col].iter_mut() {
            *value /= diagonal;
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
    let mut out = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            out.set(i, j, work[i][n + j]);
        }
    }
    out
}

/// The max-norm residual of the control DARE
/// `AᵀPA − AᵀPB(R+BᵀPB)⁻¹BᵀPA + Q − P` at `P`.
pub fn control_dare_residual(a: &Matrix, b: &Matrix, q: &Matrix, r: &Matrix, p: &Matrix) -> f64 {
    let at = a.transpose();
    let bt = b.transpose();
    let btp = bt.matmul(p).unwrap();
    let inner = add(r, &btp.matmul(b).unwrap());
    let inner_inv = inverse(&inner);
    let btpa = btp.matmul(a).unwrap();
    let atpb = btpa.transpose();
    let atpa = at.matmul(p).unwrap().matmul(a).unwrap();
    let correction = atpb.matmul(&inner_inv).unwrap().matmul(&btpa).unwrap();
    let rebuilt = add(&sub(&atpa, &correction), q);
    max_abs_diff(&rebuilt, p)
}

/// The max-norm residual of the filter DARE
/// `APAᵀ − APCᵀ(R+CPCᵀ)⁻¹CPAᵀ + Q − P` at `P`.
pub fn filter_dare_residual(a: &Matrix, c: &Matrix, q: &Matrix, r: &Matrix, p: &Matrix) -> f64 {
    let at = a.transpose();
    let ct = c.transpose();
    let apat = a.matmul(p).unwrap().matmul(&at).unwrap();
    let apct = a.matmul(p).unwrap().matmul(&ct).unwrap();
    let cpct = c.matmul(p).unwrap().matmul(&ct).unwrap();
    let inner_inv = inverse(&add(r, &cpct));
    let cpat = c.matmul(p).unwrap().matmul(&at).unwrap();
    let correction = apct.matmul(&inner_inv).unwrap().matmul(&cpat).unwrap();
    let rebuilt = add(&sub(&apat, &correction), q);
    max_abs_diff(&rebuilt, p)
}

/// The spectral radius `max |λ|` of a pole list.
pub fn spectral_radius(poles: &[Complex]) -> f64 {
    poles.iter().map(|pole| pole.abs()).fold(0.0, f64::max)
}

/// The elementwise sum `a + b`.
pub fn add(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) + b.get(i, j));
        }
    }
    out
}

/// The elementwise difference `a − b`.
pub fn sub(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) - b.get(i, j));
        }
    }
    out
}

/// The largest absolute entry of `a − b`.
pub fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            best = best.max((a.get(i, j) - b.get(i, j)).abs());
        }
    }
    best
}

/// The Euclidean norm of a state vector.
pub fn norm(x: &[f64]) -> f64 {
    x.iter().map(|value| value * value).sum::<f64>().sqrt()
}

/// One step of `A x + B u` for a scalar-input system, `u` a slice of length `m`.
pub fn step(a: &Matrix, x: &[f64]) -> Vec<f64> {
    (0..a.rows()).map(|i| (0..a.cols()).map(|j| a.get(i, j) * x[j]).sum()).collect()
}

/// `A x + B u` for column-vector `x` and input `u`.
pub fn step_input(a: &Matrix, b: &Matrix, x: &[f64], u: &[f64]) -> Vec<f64> {
    let mut out = step(a, x);
    for (i, entry) in out.iter_mut().enumerate() {
        let bu: f64 = (0..b.cols()).map(|j| b.get(i, j) * u[j]).sum();
        *entry += bu;
    }
    out
}

/// True when two matrices are equal bit-for-bit (`f64::to_bits`).
pub fn bits_equal(a: &Matrix, b: &Matrix) -> bool {
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

/// True when two pole lists are equal bit-for-bit, in order.
pub fn poles_bits_equal(a: &[Complex], b: &[Complex]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.re.to_bits() == y.re.to_bits() && x.im.to_bits() == y.im.to_bits())
}

/// Asserts two pole multisets match to `tolerance`, order-independent.
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
        assert!(distance < tolerance, "pole mismatch: want {want}, nearest at distance {distance}");
        used[index] = true;
    }
}
