//! Shared deterministic fixtures for the state-estimation integration tests.
// Shared across multiple test binaries; not every helper is used by every one.
#![allow(dead_code)]
// Explicit index loops here mirror the textbook linear-algebra formulas.
#![allow(clippy::needless_range_loop)]

use lawsynth_estimate::{Complex, Matrix};

/// The double integrator `ẋ = [[0,1],[0,0]] x + [[0],[1]] u`, a canonical
/// observable/controllable SISO plant (position–velocity). Measuring only
/// position (`C = [1, 0]`) still lets an observer reconstruct velocity.
pub fn double_integrator() -> (Matrix, Matrix, Matrix) {
    let a = Matrix::from_rows(vec![vec![0.0, 1.0], vec![0.0, 0.0]]).unwrap();
    let b = Matrix::from_rows(vec![vec![0.0], vec![1.0]]).unwrap();
    let c = Matrix::from_rows(vec![vec![1.0, 0.0]]).unwrap();
    (a, b, c)
}

/// A stable damped oscillator `ẋ = [[0,1],[-2,-3]] x`, measuring position only.
/// Eigenvalues of `A` are `{−1, −2}` (already stable), so this is a good testbed
/// for a Kalman filter reconstructing the unmeasured velocity.
pub fn damped_oscillator() -> (Matrix, Matrix, Matrix) {
    let a = Matrix::from_rows(vec![vec![0.0, 1.0], vec![-2.0, -3.0]]).unwrap();
    let b = Matrix::from_rows(vec![vec![0.0], vec![1.0]]).unwrap();
    let c = Matrix::from_rows(vec![vec![1.0, 0.0]]).unwrap();
    (a, b, c)
}

/// A diagonal 2×2 matrix.
pub fn diag2(d0: f64, d1: f64) -> Matrix {
    Matrix::from_rows(vec![vec![d0, 0.0], vec![0.0, d1]]).unwrap()
}

/// A 1×1 matrix.
pub fn scalar(value: f64) -> Matrix {
    Matrix::from_rows(vec![vec![value]]).unwrap()
}

/// Real desired poles as `Complex` values.
pub fn real_poles(values: &[f64]) -> Vec<Complex> {
    values.iter().map(|&v| Complex::real(v)).collect()
}

/// The maximum absolute entrywise difference between two matrices.
pub fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            best = best.max((a.get(i, j) - b.get(i, j)).abs());
        }
    }
    best
}

/// True when a computed pole set matches a desired real pole set (each desired
/// pole is matched to within `tol` by some achieved pole).
pub fn poles_match(achieved: &[Complex], desired: &[f64], tol: f64) -> bool {
    if achieved.len() != desired.len() {
        return false;
    }
    desired.iter().all(|&target| {
        achieved.iter().any(|pole| (pole.re - target).abs() < tol && pole.im.abs() < tol)
    })
}

/// Bit-identical comparison of two matrices via `f64::to_bits`.
pub fn bits_identical(a: &Matrix, b: &Matrix) -> bool {
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

/// The residual of the continuous **filter** algebraic Riccati equation
/// `A P + P Aᵀ − P Cᵀ R⁻¹ C P + Q`, as a max-norm. For a correct steady-state
/// Kalman covariance this is ≈ 0. `R` is required diagonal so its inverse is
/// formed directly here (no dependency on private feedback internals).
pub fn filter_care_residual(a: &Matrix, c: &Matrix, q: &Matrix, r_diag: &[f64], p: &Matrix) -> f64 {
    let n = a.rows();
    let ap = a.matmul(p).unwrap();
    let pat = p.matmul(&a.transpose()).unwrap();

    // P Cᵀ R⁻¹ C P with R⁻¹ = diag(1/r_i).
    let ct = c.transpose(); // n×p
    let mut r_inv_c = c.clone(); // p×n, will become R⁻¹ C
    for i in 0..c.rows() {
        for j in 0..c.cols() {
            r_inv_c.set(i, j, c.get(i, j) / r_diag[i]);
        }
    }
    let pct = p.matmul(&ct).unwrap(); // n×p
    let quad = pct.matmul(&r_inv_c).unwrap().matmul(p).unwrap(); // n×n

    let mut residual = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            let value = ap.get(i, j) + pat.get(i, j) - quad.get(i, j) + q.get(i, j);
            residual = residual.max(value.abs());
        }
    }
    residual
}

/// True when the symmetric matrix `p` is positive definite, tested by an
/// attempted Cholesky factorization `p = L Lᵀ` (succeeds iff every pivot is
/// strictly positive).
pub fn is_positive_definite(p: &Matrix) -> bool {
    let n = p.rows();
    if p.cols() != n {
        return false;
    }
    let mut l = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let mut sum = p.get(i, j);
            for k in 0..j {
                sum -= l[i][k] * l[j][k];
            }
            if i == j {
                if sum <= 0.0 {
                    return false;
                }
                l[i][j] = sum.sqrt();
            } else {
                l[i][j] = sum / l[j][j];
            }
        }
    }
    true
}

/// True when `p` is symmetric within `tol`.
pub fn is_symmetric(p: &Matrix, tol: f64) -> bool {
    let n = p.rows();
    if p.cols() != n {
        return false;
    }
    for i in 0..n {
        for j in i + 1..n {
            if (p.get(i, j) - p.get(j, i)).abs() > tol {
                return false;
            }
        }
    }
    true
}
