//! Parameter-covariance construction and the linear algebra the two forecast
//! methods share: the sample covariance of bootstrap replicates, a Cholesky
//! factor for Gaussian sampling, and covariance shape validation.

use lawsynth_uncertainty::CoefficientEnsemble;

use crate::error::PropagateError;

/// Sample covariance `Cov(θ)` of the bootstrap replicate coefficient vectors.
///
/// Given a [`CoefficientEnsemble`] whose `replicates` are the per-resample
/// coefficient draws (shape `[B][p]`), this returns the `p × p` unbiased sample
/// covariance
///
/// ```text
/// Cov(θ)[j][l] = (1 / (B − 1)) · Σ_b (θ_bj − θ̄_j)(θ_bl − θ̄_l),
/// ```
///
/// using the `B − 1` denominator so it matches the unbiased variance reported by
/// `lawsynth-uncertainty`'s per-term `standard_error`. This is the bridge from a
/// coefficient bootstrap straight to a forecast band: feed the result to
/// [`crate::delta_forecast`] or to a Gaussian Monte-Carlo draw.
///
/// A degenerate ensemble with fewer than two replicates has no defined sample
/// covariance; a zero matrix of the correct shape is returned in that case (the
/// bootstrap always produces at least two replicates, so this is only reachable
/// for hand-built ensembles).
pub fn covariance_from_ensemble(ensemble: &CoefficientEnsemble) -> Vec<Vec<f64>> {
    let features = ensemble.features();
    let replicates = &ensemble.replicates;
    if replicates.len() < 2 {
        return vec![vec![0.0; features]; features];
    }
    let count = replicates.len();
    let mut means = vec![0.0; features];
    for draw in replicates {
        for (mean, value) in means.iter_mut().zip(draw) {
            *mean += value;
        }
    }
    for mean in &mut means {
        *mean /= count as f64;
    }

    let mut covariance = vec![vec![0.0; features]; features];
    for draw in replicates {
        for j in 0..features {
            let centered_j = draw[j] - means[j];
            for l in 0..features {
                covariance[j][l] += centered_j * (draw[l] - means[l]);
            }
        }
    }
    let denominator = (count - 1) as f64;
    for row in &mut covariance {
        for value in row {
            *value /= denominator;
        }
    }
    covariance
}

/// Validates that `covariance` is a finite, square `expected × expected` matrix.
pub(crate) fn validate_covariance(
    covariance: &[Vec<f64>],
    expected: usize,
) -> Result<(), PropagateError> {
    if covariance.len() != expected {
        return Err(PropagateError::CovarianceDimensionMismatch {
            expected,
            actual: covariance.len(),
        });
    }
    for row in covariance {
        if row.len() != expected {
            return Err(PropagateError::CovarianceNotSquare {
                rows: covariance.len(),
                cols: row.len(),
            });
        }
        if row.iter().any(|value| !value.is_finite()) {
            return Err(PropagateError::NonFiniteValue);
        }
    }
    Ok(())
}

/// Lower-triangular Cholesky factor `L` with `L · Lᵀ = matrix`.
///
/// Uses the Cholesky–Banachiewicz recurrence in a fixed accumulation order. A
/// non-positive pivot means the matrix is not positive definite, so no real
/// factor exists and [`PropagateError::NotPositiveSemiDefinite`] is returned
/// rather than a fabricated (complex or `NaN`) result.
pub(crate) fn cholesky(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, PropagateError> {
    let n = matrix.len();
    let mut lower = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let dot: f64 = lower[i][..j].iter().zip(&lower[j][..j]).map(|(a, b)| a * b).sum();
            let sum = matrix[i][j] - dot;
            if i == j {
                if sum <= 0.0 {
                    return Err(PropagateError::NotPositiveSemiDefinite);
                }
                lower[i][j] = sum.sqrt();
            } else {
                lower[i][j] = sum / lower[j][j];
            }
        }
    }
    Ok(lower)
}

/// Returns `L · z` for a lower-triangular `L` and a vector `z`, in fixed order.
pub(crate) fn lower_triangular_matvec(lower: &[Vec<f64>], z: &[f64]) -> Vec<f64> {
    lower.iter().map(|row| row.iter().zip(z).map(|(l, value)| l * value).sum()).collect()
}

#[cfg(test)]
mod tests {
    use lawsynth_uncertainty::{CoefficientEnsemble, TermUncertainty};

    use super::*;

    fn term() -> TermUncertainty {
        TermUncertainty {
            mean: 0.0,
            standard_error: 0.0,
            lower: 0.0,
            upper: 0.0,
            inclusion_probability: 1.0,
        }
    }

    #[test]
    fn covariance_matches_hand_computed_value() {
        // Two features, three replicates.
        // Column 0: [1, 2, 3] -> mean 2, unbiased var ((1)+(0)+(1))/2 = 1.
        // Column 1: [2, 4, 6] -> mean 4, unbiased var ((4)+(0)+(4))/2 = 4.
        // Covariance(0,1) = ((1*2)+(0)+(1*2))/2 = 2.
        let ensemble = CoefficientEnsemble {
            terms: vec![term(), term()],
            replicates: vec![vec![1.0, 2.0], vec![2.0, 4.0], vec![3.0, 6.0]],
            confidence: 0.95,
        };
        let covariance = covariance_from_ensemble(&ensemble);
        assert!((covariance[0][0] - 1.0).abs() < 1e-12);
        assert!((covariance[1][1] - 4.0).abs() < 1e-12);
        assert!((covariance[0][1] - 2.0).abs() < 1e-12);
        assert_eq!(covariance[0][1], covariance[1][0]);
    }

    #[test]
    fn cholesky_reconstructs_the_matrix() {
        let matrix = vec![vec![4.0, 2.0], vec![2.0, 3.0]];
        let lower = cholesky(&matrix).unwrap();
        // Reconstruct L Lᵀ and compare.
        for i in 0..2 {
            for j in 0..2 {
                let entry: f64 = (0..2).map(|k| lower[i][k] * lower[j][k]).sum();
                assert!((entry - matrix[i][j]).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn cholesky_rejects_indefinite_matrix() {
        let matrix = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        assert_eq!(cholesky(&matrix), Err(PropagateError::NotPositiveSemiDefinite));
    }

    #[test]
    fn validate_covariance_flags_wrong_dimension() {
        let covariance = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        assert_eq!(
            validate_covariance(&covariance, 3),
            Err(PropagateError::CovarianceDimensionMismatch { expected: 3, actual: 2 })
        );
    }
}
