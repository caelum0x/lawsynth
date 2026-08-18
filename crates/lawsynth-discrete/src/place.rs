//! Discrete Luenberger observer by dual pole placement in the z-plane.
//!
//! For an observable single-output pair `(A, C)` (with `C` of shape `1 × n`),
//! the error poles of `A − LC` are placed exactly at chosen z-plane locations by
//! the dual of Ackermann's formula:
//!
//! ```text
//! L = p(A) · O⁻¹ · eₙ,
//! ```
//!
//! where `O = [C; CA; …; CAⁿ⁻¹]` is the observability matrix, `eₙ = [0 … 0 1]ᵀ`,
//! and `p` is the monic desired characteristic polynomial `∏(z − λᵢ)`. This is
//! the transpose of the control Ackermann formula applied to `(Aᵀ, Cᵀ)`, so
//! `A − LC` and `(Aᵀ − CᵀLᵀ)` share the requested spectrum. The placement
//! formula is identical in continuous and discrete time; only the *meaning* of a
//! "good" pole differs — for a discrete observer the targets must lie inside the
//! unit circle for `x̂ → x`.

use lawsynth_koopman::{Complex, Matrix, eigen};

use crate::error::DiscreteError;
use crate::linalg;
use crate::observer::{DiscreteObserver, ObserverMethod};

/// Places the discrete observer error poles of `(A, C)` at `desired` z-plane
/// locations, returning the gain `L` (shape `n × 1`) and the achieved error
/// poles of `A − LC`.
///
/// # Errors
/// - [`DiscreteError::NonSquare`] / [`DiscreteError::EmptyMatrix`] for a bad `A`.
/// - [`DiscreteError::MultiOutput`] if `C` is not `1 × n`.
/// - [`DiscreteError::ShapeMismatch`] if `C`'s width is not the order of `A`.
/// - [`DiscreteError::NonFiniteValue`] for a non-finite entry.
/// - [`DiscreteError::PoleCountMismatch`] if `desired.len() != n`.
/// - [`DiscreteError::NonRealDesignPoles`] if the poles are not closed under
///   conjugation (so the gain would be complex).
/// - [`DiscreteError::Unobservable`] if the observability matrix is singular.
pub fn discrete_observer_from_poles(
    a: &Matrix,
    c: &Matrix,
    desired: &[Complex],
) -> Result<DiscreteObserver, DiscreteError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(DiscreteError::NonSquare);
    }
    if n == 0 {
        return Err(DiscreteError::EmptyMatrix);
    }
    if c.rows() != 1 {
        return Err(DiscreteError::MultiOutput);
    }
    if c.cols() != n {
        return Err(DiscreteError::ShapeMismatch);
    }
    if !linalg::is_finite(a) || !linalg::is_finite(c) {
        return Err(DiscreteError::NonFiniteValue);
    }
    if desired.len() != n {
        return Err(DiscreteError::PoleCountMismatch);
    }

    let coefficients = real_monic_polynomial(desired)?;
    let observability = observability_matrix(a, c)?;
    let observability_inv =
        linalg::invert(&observability).map_err(|_| DiscreteError::Unobservable)?;

    // p(A) = Σ cₖ Aᵏ, with A⁰ = I and the leading (monic) coefficient cₙ = 1.
    let poly_of_a = matrix_polynomial(a, &coefficients);

    // eₙ = last standard basis vector, as an n × 1 matrix.
    let mut basis = Matrix::zeros(n, 1);
    basis.set(n - 1, 0, 1.0);

    // L = p(A) · O⁻¹ · eₙ.
    let l = linalg::mm3(&poly_of_a, &observability_inv, &basis)?;

    let error_dynamics = linalg::sub(a, &linalg::mm(&l, c)?);
    let error_poles = eigen(&error_dynamics)?.values;

    Ok(DiscreteObserver { l, error_poles, p: None, method: ObserverMethod::PolePlacement })
}

/// Builds the observability matrix `O = [C; CA; …; CAⁿ⁻¹]` (`n × n` for a single
/// output), stacking `C Aᵏ` as successive rows.
fn observability_matrix(a: &Matrix, c: &Matrix) -> Result<Matrix, DiscreteError> {
    let n = a.rows();
    let mut out = Matrix::zeros(n, n);
    let mut row = c.clone(); // 1 × n, currently C A⁰
    for k in 0..n {
        for j in 0..n {
            out.set(k, j, row.get(0, j));
        }
        if k + 1 < n {
            row = linalg::mm(&row, a)?; // C Aᵏ⁺¹
        }
    }
    Ok(out)
}

/// Expands the monic polynomial `∏(z − λᵢ)` and returns its real coefficients
/// `[c₀, c₁, …, cₙ₋₁, cₙ = 1]`, verifying the roots are closed under conjugation
/// (imaginary parts cancel to within tolerance).
fn real_monic_polynomial(roots: &[Complex]) -> Result<Vec<f64>, DiscreteError> {
    // Coefficients low-to-high degree; start with the constant polynomial 1.
    let mut coefficients = vec![Complex::ONE];
    for &root in roots {
        let mut next = vec![Complex::ZERO; coefficients.len() + 1];
        for (index, &coefficient) in coefficients.iter().enumerate() {
            // Multiply by (z − root): shift up by one, subtract root·coeff.
            next[index + 1] = next[index + 1].add(coefficient);
            next[index] = next[index].sub(coefficient.mul(root));
        }
        coefficients = next;
    }

    let scale = coefficients.iter().map(|coefficient| coefficient.abs()).fold(0.0, f64::max);
    let tolerance = 1e-9 * (1.0 + scale);
    let mut real = Vec::with_capacity(coefficients.len());
    for coefficient in &coefficients {
        if coefficient.im.abs() > tolerance {
            return Err(DiscreteError::NonRealDesignPoles);
        }
        real.push(coefficient.re);
    }
    Ok(real)
}

/// Evaluates the matrix polynomial `Σ cₖ Aᵏ` (with `A⁰ = I`).
fn matrix_polynomial(a: &Matrix, coefficients: &[f64]) -> Matrix {
    let n = a.rows();
    let mut result = Matrix::zeros(n, n);
    let mut power = Matrix::identity(n); // A⁰
    for (degree, &coefficient) in coefficients.iter().enumerate() {
        for i in 0..n {
            for j in 0..n {
                result.set(i, j, result.get(i, j) + coefficient * power.get(i, j));
            }
        }
        if degree + 1 < coefficients.len() {
            power = a.matmul(&power).expect("square power stays square");
        }
    }
    result
}
