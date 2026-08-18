//! Single-input pole placement by Ackermann's formula.
//!
//! Given a controllable pair `(A, b)` with a single input and a set of desired
//! closed-loop poles, Ackermann's formula produces the unique gain
//! `K = [0 … 0 1] · C⁻¹ · p(A)`, where `C = [b, Ab, …, Aⁿ⁻¹b]` is the
//! controllability matrix and `p` is the desired characteristic polynomial. The
//! poles must be closed under conjugation so `p` — and therefore `K` — is real.

use lawsynth_koopman::{Complex, Matrix, eigen};

use crate::error::FeedbackError;
use crate::gain::Gain;
use crate::linalg;

/// Designs a single-input state-feedback gain placing the closed-loop poles of
/// `A − b K` at `desired`.
///
/// # Errors
/// - [`FeedbackError::MultiInput`] if `b` is not `n × 1`.
/// - [`FeedbackError::PoleCountMismatch`] if `desired.len() != n`.
/// - [`FeedbackError::NonRealDesignPoles`] if the poles are not conjugate-closed.
/// - [`FeedbackError::Uncontrollable`] if `[b, Ab, …, Aⁿ⁻¹b]` is singular.
pub fn place_poles(a: &Matrix, b: &Matrix, desired: &[Complex]) -> Result<Gain, FeedbackError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(FeedbackError::NonSquare);
    }
    if n == 0 {
        return Err(FeedbackError::EmptyMatrix);
    }
    if !linalg::is_finite(a) || !linalg::is_finite(b) {
        return Err(FeedbackError::NonFiniteValue);
    }
    if b.rows() != n {
        return Err(FeedbackError::ShapeMismatch);
    }
    if b.cols() != 1 {
        return Err(FeedbackError::MultiInput);
    }
    if desired.len() != n {
        return Err(FeedbackError::PoleCountMismatch);
    }

    // Desired characteristic polynomial (monic, degree n) with real coefficients.
    let coefficients = real_characteristic_polynomial(desired)?;

    // p(A) evaluated as a matrix polynomial.
    let p_of_a = matrix_polynomial(a, &coefficients)?;

    // Controllability matrix and its inverse; singular ⇒ uncontrollable.
    let controllability = controllability_matrix(a, b)?;
    let controllability_inverse =
        linalg::invert(&controllability).map_err(|error| match error {
            FeedbackError::SingularSystem => FeedbackError::Uncontrollable,
            other => other,
        })?;

    // K = eₙᵀ · C⁻¹ · p(A): take the last row of C⁻¹ and multiply by p(A).
    let mut last_row = Matrix::zeros(1, n);
    for j in 0..n {
        last_row.set(0, j, controllability_inverse.get(n - 1, j));
    }
    let k = linalg::mm(&last_row, &p_of_a)?;

    let closed_loop = linalg::sub(a, &linalg::mm(b, &k)?);
    let achieved_poles = eigen(&closed_loop)?.values;

    Ok(Gain { k, achieved_poles, p: None })
}

/// Expands `∏ (s − λᵢ)` into monic real coefficients `[c₀, …, cₙ₋₁, 1]`.
///
/// The product is accumulated in complex arithmetic; if the desired poles are
/// closed under conjugation the imaginary parts cancel, so a residual imaginary
/// component above tolerance signals [`FeedbackError::NonRealDesignPoles`].
fn real_characteristic_polynomial(roots: &[Complex]) -> Result<Vec<f64>, FeedbackError> {
    let n = roots.len();
    let mut coefficients = vec![Complex::ZERO; n + 1];
    coefficients[0] = Complex::ONE;
    let mut max_root = 0.0_f64;

    for (degree, &root) in roots.iter().enumerate() {
        max_root = max_root.max(root.abs());
        // Multiply the current polynomial by (s − root): new[k] = old[k−1] − root·old[k].
        for k in (0..=degree + 1).rev() {
            let lower = if k >= 1 { coefficients[k - 1] } else { Complex::ZERO };
            let scaled = root.mul(coefficients[k]);
            coefficients[k] = lower.sub(scaled);
        }
    }

    let tolerance = 1e-9 * (1.0 + max_root).powi(n as i32);
    if coefficients.iter().any(|value| value.im.abs() > tolerance) {
        return Err(FeedbackError::NonRealDesignPoles);
    }

    Ok(coefficients.iter().map(|value| value.re).collect())
}

/// Evaluates `p(A) = Σ cₖ Aᵏ` for real coefficients `c`.
fn matrix_polynomial(a: &Matrix, coefficients: &[f64]) -> Result<Matrix, FeedbackError> {
    let n = a.rows();
    let mut result = Matrix::zeros(n, n);
    let mut power = Matrix::identity(n); // A⁰
    let last = coefficients.len() - 1;
    for (index, &coefficient) in coefficients.iter().enumerate() {
        for i in 0..n {
            for j in 0..n {
                result.set(i, j, result.get(i, j) + coefficient * power.get(i, j));
            }
        }
        if index < last {
            power = linalg::mm(&power, a)?;
        }
    }
    Ok(result)
}

/// Builds the controllability matrix `C = [b, Ab, …, Aⁿ⁻¹b]` (columns).
fn controllability_matrix(a: &Matrix, b: &Matrix) -> Result<Matrix, FeedbackError> {
    let n = a.rows();
    let mut columns = Matrix::zeros(n, n);
    let mut vector: Vec<f64> = (0..n).map(|i| b.get(i, 0)).collect();
    for k in 0..n {
        for (i, &value) in vector.iter().enumerate() {
            columns.set(i, k, value);
        }
        vector = a.mat_vec(&vector).map_err(|_| FeedbackError::ShapeMismatch)?;
    }
    Ok(columns)
}
