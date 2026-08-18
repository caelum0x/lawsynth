//! The public balanced-truncation API: order selection and model construction.

use lawsynth_koopman::{Matrix, eigen};

use crate::balance::balancing_transform;
use crate::error::ModelReduceError;
use crate::linalg::mm;

/// How to choose the reduced order `k`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReductionSpec {
    /// Keep exactly `k` states (the `k` largest Hankel singular values).
    Order(usize),
    /// Keep the fewest states whose **discarded** Hankel-singular-value energy is
    /// at most this fraction of the total, i.e. the smallest `k` with
    /// `Σ_{i>k} σ_i ≤ tol · Σ_i σ_i`. Must be finite and in `[0, 1)`.
    EnergyTolerance(f64),
}

/// A reduced-order linear model produced by balanced truncation.
#[derive(Clone, Debug, PartialEq)]
pub struct ReducedModel {
    /// The `k × k` reduced state matrix `Aᵣ`.
    pub a: Matrix,
    /// The `k × m` reduced input matrix `Bᵣ`.
    pub b: Matrix,
    /// The `p × k` reduced output matrix `Cᵣ`.
    pub c: Matrix,
    /// **All** `n` Hankel singular values of the full system, non-increasing.
    pub hankel_singular_values: Vec<f64>,
    /// The retained order `k` (`= a.rows()`).
    pub order: usize,
}

impl ReducedModel {
    /// The a priori H∞ error bound `‖G − Gᵣ‖∞ ≤ 2 · Σ_{i=k+1}^n σ_i`.
    ///
    /// This is the standard balanced-truncation bound, exact in infinite-precision
    /// arithmetic; finite-precision balancing adds a small extra error.
    pub fn error_bound(&self) -> f64 {
        2.0 * self.hankel_singular_values[self.order..].iter().sum::<f64>()
    }
}

/// The state dimension after validating the `(A, B, C)` shapes.
fn validated_dimension(a: &Matrix, b: &Matrix, c: &Matrix) -> Result<usize, ModelReduceError> {
    let n = a.rows();
    if n == 0 || a.cols() == 0 || b.cols() == 0 || c.cols() == 0 {
        return Err(ModelReduceError::EmptyMatrix);
    }
    if a.cols() != n {
        return Err(ModelReduceError::NonSquareState);
    }
    if b.rows() != n {
        return Err(ModelReduceError::InputDimensionMismatch);
    }
    if c.cols() != n {
        return Err(ModelReduceError::OutputDimensionMismatch);
    }
    Ok(n)
}

/// Returns `NotStable` unless every eigenvalue of `A` has strictly negative real
/// part (a Hurwitz matrix), the precondition for the gramians to exist.
fn ensure_hurwitz(a: &Matrix) -> Result<(), ModelReduceError> {
    let spectrum = eigen(a).map_err(|_| ModelReduceError::NoConvergence)?;
    if spectrum.values.iter().any(|value| value.re >= 0.0) {
        return Err(ModelReduceError::NotStable);
    }
    Ok(())
}

/// Chooses the retained order from the spec and the Hankel singular values.
fn select_order(sigma: &[f64], spec: &ReductionSpec, n: usize) -> Result<usize, ModelReduceError> {
    match *spec {
        ReductionSpec::Order(k) => {
            if k == 0 || k > n {
                Err(ModelReduceError::InvalidOrder)
            } else {
                Ok(k)
            }
        }
        ReductionSpec::EnergyTolerance(tol) => {
            if !tol.is_finite() || !(0.0..1.0).contains(&tol) {
                return Err(ModelReduceError::InvalidTolerance);
            }
            let total: f64 = sigma.iter().sum();
            // Tail energy Σ_{i>k} σ_i is non-increasing in k; take the smallest k
            // whose tail fits under the tolerance. k = n (tail 0) always works.
            for candidate in 1..=n {
                let tail: f64 = sigma[candidate..].iter().sum();
                if tail <= tol * total {
                    return Ok(candidate);
                }
            }
            Ok(n)
        }
    }
}

/// Computes the Hankel singular values of a stable realization `(A, B, C)`.
///
/// These are the key diagnostic of balanced truncation: the square roots of the
/// eigenvalues of `Wc Wo`, in non-increasing order. A large gap between `σ_k` and
/// `σ_{k+1}` marks a natural truncation order.
pub fn hankel_singular_values(
    a: &Matrix,
    b: &Matrix,
    c: &Matrix,
) -> Result<Vec<f64>, ModelReduceError> {
    validated_dimension(a, b, c)?;
    ensure_hurwitz(a)?;
    Ok(balancing_transform(a, b, c)?.sigma)
}

/// Reduces a stable linear model `ẋ = A x + B u`, `y = C x` by balanced
/// truncation to a lower order chosen by `spec`.
///
/// The full realization is transformed into balanced coordinates (where both
/// gramians equal `diag(σ)`), then the states with the smallest Hankel singular
/// values are dropped:
///
/// ```text
/// Aᵣ = (T⁻¹ A T)[0..k, 0..k]
/// Bᵣ = (T⁻¹ B)[0..k, :]
/// Cᵣ = (C T)[:, 0..k]
/// ```
///
/// Requires `A` to be Hurwitz; returns [`ModelReduceError::NotStable`] otherwise.
pub fn balanced_truncation(
    a: &Matrix,
    b: &Matrix,
    c: &Matrix,
    spec: &ReductionSpec,
) -> Result<ReducedModel, ModelReduceError> {
    let n = validated_dimension(a, b, c)?;
    ensure_hurwitz(a)?;

    let balancing = balancing_transform(a, b, c)?;
    let k = select_order(&balancing.sigma, spec, n)?;

    // Full balanced realization, then truncate to the leading k states.
    let a_balanced = mm(&mm(&balancing.t_inv, a)?, &balancing.t)?;
    let b_balanced = mm(&balancing.t_inv, b)?;
    let c_balanced = mm(c, &balancing.t)?;

    let a_reduced = top_left_block(&a_balanced, k);
    let b_reduced = top_rows(&b_balanced, k);
    let c_reduced = left_cols(&c_balanced, k);

    Ok(ReducedModel {
        a: a_reduced,
        b: b_reduced,
        c: c_reduced,
        hankel_singular_values: balancing.sigma,
        order: k,
    })
}

/// The leading `k × k` block of a square matrix.
fn top_left_block(a: &Matrix, k: usize) -> Matrix {
    let mut out = Matrix::zeros(k, k);
    for i in 0..k {
        for j in 0..k {
            out.set(i, j, a.get(i, j));
        }
    }
    out
}

/// The first `k` rows of a matrix.
fn top_rows(a: &Matrix, k: usize) -> Matrix {
    let mut out = Matrix::zeros(k, a.cols());
    for i in 0..k {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j));
        }
    }
    out
}

/// The first `k` columns of a matrix.
fn left_cols(a: &Matrix, k: usize) -> Matrix {
    let mut out = Matrix::zeros(a.rows(), k);
    for i in 0..a.rows() {
        for j in 0..k {
            out.set(i, j, a.get(i, j));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_system() -> (Matrix, Matrix, Matrix) {
        let a = Matrix::from_rows(vec![vec![-1.0, 0.0], vec![0.0, -2.0]]).unwrap();
        let b = Matrix::from_rows(vec![vec![1.0], vec![1.0]]).unwrap();
        let c = Matrix::from_rows(vec![vec![1.0, 1.0]]).unwrap();
        (a, b, c)
    }

    #[test]
    fn order_spec_is_validated() {
        let sigma = vec![1.0, 0.5, 0.1];
        assert_eq!(
            select_order(&sigma, &ReductionSpec::Order(0), 3),
            Err(ModelReduceError::InvalidOrder)
        );
        assert_eq!(
            select_order(&sigma, &ReductionSpec::Order(4), 3),
            Err(ModelReduceError::InvalidOrder)
        );
        assert_eq!(select_order(&sigma, &ReductionSpec::Order(2), 3), Ok(2));
    }

    #[test]
    fn energy_tolerance_picks_the_smallest_sufficient_order() {
        let sigma = vec![1.0, 0.5, 0.01]; // total 1.51
        // Dropping the last state discards 0.01/1.51 ≈ 0.66% of the energy.
        assert_eq!(select_order(&sigma, &ReductionSpec::EnergyTolerance(0.01), 3), Ok(2));
        // A zero tolerance keeps everything.
        assert_eq!(select_order(&sigma, &ReductionSpec::EnergyTolerance(0.0), 3), Ok(3));
        // A generous tolerance keeps only the dominant state.
        assert_eq!(select_order(&sigma, &ReductionSpec::EnergyTolerance(0.5), 3), Ok(1));
    }

    #[test]
    fn tolerance_is_range_checked() {
        let sigma = vec![1.0];
        assert_eq!(
            select_order(&sigma, &ReductionSpec::EnergyTolerance(1.0), 1),
            Err(ModelReduceError::InvalidTolerance)
        );
        assert_eq!(
            select_order(&sigma, &ReductionSpec::EnergyTolerance(-0.1), 1),
            Err(ModelReduceError::InvalidTolerance)
        );
    }

    #[test]
    fn full_order_truncation_preserves_dimension() {
        let (a, b, c) = sample_system();
        let reduced = balanced_truncation(&a, &b, &c, &ReductionSpec::Order(2)).unwrap();
        assert_eq!(reduced.order, 2);
        assert_eq!((reduced.a.rows(), reduced.a.cols()), (2, 2));
        assert_eq!(reduced.error_bound(), 0.0);
    }
}
