//! Deterministic bootstrap confidence intervals for discovered sparse coefficients.
//!
//! Given a candidate library matrix `Θ` (row-major) and a target vector `ẋ`,
//! this module re-fits the sequentially-thresholded least-squares (STLSQ)
//! solver on many deterministic resamples and reports, per candidate term:
//! a mean, a standard error (spread of the bootstrap distribution), a
//! percentile confidence interval, and — the headline honest signal — an
//! **inclusion probability**: the fraction of resamples in which the term
//! survived sparsity (kept a non-zero coefficient).
//!
//! Determinism is total: resample `b` draws its indices from a SplitMix64
//! state derived solely from `(seed, b)`, so the ensemble is bit-reproducible
//! and independent of iteration order or thread count.

use lawsynth_sparse::{RegressionProblem, SparseConfig, SparseError, stlsq};

use crate::bootstrap::{next_index, next_u64};
use crate::{Samples, UncertaintyError, percentile};

/// How rows are resampled to build each bootstrap replicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResampleMode {
    /// Case/pairs bootstrap: resample `(row_of_Θ, target)` pairs with
    /// replacement. Assumes rows are exchangeable.
    Cases,
    /// Residual bootstrap: fit once, then resample the fitted residuals with
    /// replacement and rebuild synthetic targets `ŷ + r*`. Holds the design
    /// matrix fixed and assumes homoscedastic, exchangeable residuals.
    Residual,
}

/// Controls the deterministic coefficient bootstrap.
#[derive(Clone, Debug, PartialEq)]
pub struct BootstrapCoefficientConfig {
    /// Number of bootstrap resamples `B`.
    pub resamples: usize,
    /// Fixed root seed. Replicate `b` is seeded from `(seed, b)`.
    pub seed: u64,
    /// Two-sided confidence level for the percentile interval, in `(0, 1)`.
    pub confidence: f64,
    /// Resampling scheme.
    pub mode: ResampleMode,
    /// Sparse-fit settings (threshold, ridge, iterations) applied to every refit.
    pub sparse: SparseConfig,
}

impl Default for BootstrapCoefficientConfig {
    fn default() -> Self {
        Self {
            resamples: 200,
            seed: 0x4c53_5f43_4f45_4646,
            confidence: 0.95,
            mode: ResampleMode::Cases,
            sparse: SparseConfig::default(),
        }
    }
}

impl BootstrapCoefficientConfig {
    /// At least two resamples are required to form a variance and a two-sided
    /// percentile interval.
    pub fn validate(&self) -> Result<(), UncertaintyError> {
        if self.resamples < 2 {
            return Err(UncertaintyError::InvalidBootstrapConfig);
        }
        if !self.confidence.is_finite() || self.confidence <= 0.0 || self.confidence >= 1.0 {
            return Err(UncertaintyError::InvalidConfidence(self.confidence));
        }
        Ok(())
    }
}

/// Per-term uncertainty summary aggregated across the bootstrap ensemble.
#[derive(Clone, Debug, PartialEq)]
pub struct TermUncertainty {
    /// Mean coefficient across all resamples (including zeros).
    pub mean: f64,
    /// Standard error: the (unbiased) standard deviation of the bootstrap
    /// distribution of the coefficient. This is the spread of the estimator,
    /// not the standard error of the mean of the resamples.
    pub standard_error: f64,
    /// Lower percentile-interval endpoint (the `α/2` empirical quantile).
    pub lower: f64,
    /// Upper percentile-interval endpoint (the `1 − α/2` empirical quantile).
    pub upper: f64,
    /// Fraction of resamples in which this term kept a non-zero coefficient
    /// (survived sparsity). The honest "how sure are we this term belongs?"
    /// signal.
    pub inclusion_probability: f64,
}

/// The full bootstrap ensemble: per-term summaries plus the raw replicate draws.
#[derive(Clone, Debug, PartialEq)]
pub struct CoefficientEnsemble {
    /// One summary per candidate term, in library column order.
    pub terms: Vec<TermUncertainty>,
    /// Raw coefficient draws, shape `[resamples][features]`, in replicate order.
    /// Replicate `b` is fully determined by `(seed, b)`, so a prefix of this
    /// vector is stable as `resamples` grows.
    pub replicates: Vec<Vec<f64>>,
    /// The confidence level used for the intervals.
    pub confidence: f64,
}

impl CoefficientEnsemble {
    /// Number of candidate terms (library columns).
    pub fn features(&self) -> usize {
        self.terms.len()
    }

    /// Number of bootstrap resamples.
    pub fn resamples(&self) -> usize {
        self.replicates.len()
    }
}

/// Bootstrap per-coefficient confidence intervals and inclusion probabilities.
///
/// `theta` is the row-major library matrix `Θ` (each inner slice is one
/// observation's feature row); `target` is the aligned response `ẋ`. The
/// returned [`CoefficientEnsemble`] reports, per column, the mean, standard
/// error, percentile interval, and inclusion probability.
pub fn bootstrap_coefficients(
    theta: &[Vec<f64>],
    target: &[f64],
    config: &BootstrapCoefficientConfig,
) -> Result<CoefficientEnsemble, UncertaintyError> {
    config.validate()?;
    validate_design(theta, target)?;
    let observations = target.len();
    let features = theta[0].len();

    // Residual mode fits once up front to obtain fitted values and residuals.
    let residual_base = match config.mode {
        ResampleMode::Cases => None,
        ResampleMode::Residual => {
            let solution = fit(theta, target, &config.sparse)?;
            let fitted: Vec<f64> =
                theta.iter().map(|row| dot(row, &solution.coefficients)).collect();
            let residuals: Vec<f64> = target.iter().zip(&fitted).map(|(y, f)| y - f).collect();
            Some((fitted, residuals))
        }
    };

    let mut replicates = Vec::with_capacity(config.resamples);
    for replicate in 0..config.resamples {
        let mut state = replicate_state(config.seed, replicate);
        let coefficients = match &residual_base {
            None => {
                let mut rows = Vec::with_capacity(observations);
                let mut resampled_target = Vec::with_capacity(observations);
                for _ in 0..observations {
                    let index = next_index(&mut state, observations);
                    rows.push(theta[index].clone());
                    resampled_target.push(target[index]);
                }
                fit(&rows, &resampled_target, &config.sparse)?.coefficients
            }
            Some((fitted, residuals)) => {
                let mut resampled_target = Vec::with_capacity(observations);
                for &base in fitted {
                    let index = next_index(&mut state, observations);
                    resampled_target.push(base + residuals[index]);
                }
                fit(theta, &resampled_target, &config.sparse)?.coefficients
            }
        };
        replicates.push(coefficients);
    }

    let terms = aggregate(&replicates, features, config.confidence)?;
    Ok(CoefficientEnsemble { terms, replicates, confidence: config.confidence })
}

/// Derives an independent SplitMix64 state for replicate `b` from `(seed, b)`.
///
/// Because the state depends only on `(seed, b)`, each replicate is reproducible
/// regardless of the order in which replicates are computed.
fn replicate_state(seed: u64, replicate: usize) -> u64 {
    let mut state = seed.wrapping_add((replicate as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
    // One mixing step decorrelates adjacent `(seed, b)` pairs before drawing.
    next_u64(&mut state)
}

fn dot(row: &[f64], coefficients: &[f64]) -> f64 {
    row.iter().zip(coefficients).map(|(x, w)| x * w).sum()
}

fn fit(
    rows: &[Vec<f64>],
    target: &[f64],
    config: &SparseConfig,
) -> Result<lawsynth_sparse::SparseSolution, UncertaintyError> {
    let problem = RegressionProblem::new(rows.to_vec(), target.to_vec()).map_err(map_sparse)?;
    stlsq(&problem, config).map_err(map_sparse)
}

fn map_sparse(error: SparseError) -> UncertaintyError {
    match error {
        SparseError::NonFiniteValue => UncertaintyError::NonFiniteValue,
        other => UncertaintyError::FitFailure(other.to_string()),
    }
}

fn validate_design(theta: &[Vec<f64>], target: &[f64]) -> Result<(), UncertaintyError> {
    let Some(first) = theta.first() else {
        return Err(UncertaintyError::EmptyInput);
    };
    if first.is_empty() {
        return Err(UncertaintyError::EmptyInput);
    }
    if theta.len() != target.len() {
        return Err(UncertaintyError::DimensionMismatch {
            expected: theta.len(),
            actual: target.len(),
        });
    }
    let width = first.len();
    if theta.iter().any(|row| row.len() != width) {
        return Err(UncertaintyError::DimensionMismatch {
            expected: width,
            actual: theta.iter().map(Vec::len).find(|&len| len != width).unwrap_or(width),
        });
    }
    if theta.iter().flatten().chain(target).any(|value| !value.is_finite()) {
        return Err(UncertaintyError::NonFiniteValue);
    }
    Ok(())
}

/// Aggregates raw replicate draws into per-term summaries.
///
/// The confidence interval uses the **percentile method**: the lower endpoint
/// is the `α/2` empirical quantile and the upper endpoint the `1 − α/2`
/// quantile, where `α = 1 − confidence`. Quantiles use linear interpolation
/// between the two nearest order statistics at rank `p·(n − 1)` (the R type-7
/// rule), with values placed in a total ordering — the same deterministic rule
/// as [`crate::percentile`].
pub(crate) fn aggregate(
    replicates: &[Vec<f64>],
    features: usize,
    confidence: f64,
) -> Result<Vec<TermUncertainty>, UncertaintyError> {
    if replicates.len() < 2 {
        return Err(UncertaintyError::InsufficientResamples);
    }
    let tail = (1.0 - confidence) / 2.0;
    let mut summaries = Vec::with_capacity(features);
    for column in 0..features {
        let values: Vec<f64> = replicates.iter().map(|draw| draw[column]).collect();
        let samples = Samples::new(values.clone())?;
        let mean = samples.mean();
        let standard_error = samples.variance()?.sqrt();
        let lower = percentile(&values, tail)?;
        let upper = percentile(&values, 1.0 - tail)?;
        let survivors = values.iter().filter(|value| is_active(**value)).count();
        let inclusion_probability = survivors as f64 / values.len() as f64;
        summaries.push(TermUncertainty {
            mean,
            standard_error,
            lower,
            upper,
            inclusion_probability,
        });
    }
    Ok(summaries)
}

/// A term survived sparsity when STLSQ left its coefficient non-zero.
///
/// STLSQ sets thresholded terms to exactly `0.0` and leaves survivors at their
/// least-squares value, so exact-zero testing is the honest inclusion signal.
fn is_active(value: f64) -> bool {
    // STLSQ writes an exact 0.0 for pruned terms; anything else survived.
    value.abs() > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_matches_hand_computed_quantiles() {
        // Single term, five replicates: [0, 1, 2, 3, 4].
        let replicates: Vec<Vec<f64>> =
            [0.0, 1.0, 2.0, 3.0, 4.0].into_iter().map(|value| vec![value]).collect();

        // confidence 0.8 -> tail 0.1. Lower rank = 0.1*4 = 0.4 -> 0 + 0.4*(1-0) = 0.4.
        // Upper rank = 0.9*4 = 3.6 -> 3 + 0.6*(4-3) = 3.6.
        let summary = &aggregate(&replicates, 1, 0.8).unwrap()[0];
        assert!((summary.lower - 0.4).abs() < 1e-12);
        assert!((summary.upper - 3.6).abs() < 1e-12);
        assert!((summary.mean - 2.0).abs() < 1e-12);
        // Unbiased variance = (4+1+0+1+4)/4 = 2.5 -> std = sqrt(2.5).
        assert!((summary.standard_error - 2.5_f64.sqrt()).abs() < 1e-12);
        // Four of five draws are non-zero.
        assert!((summary.inclusion_probability - 0.8).abs() < 1e-12);
    }

    #[test]
    fn aggregate_confidence_half_hits_order_statistics_exactly() {
        let replicates: Vec<Vec<f64>> =
            [0.0, 1.0, 2.0, 3.0, 4.0].into_iter().map(|value| vec![value]).collect();
        // confidence 0.5 -> tail 0.25. Lower rank = 0.25*4 = 1.0 -> exactly 1.0.
        // Upper rank = 0.75*4 = 3.0 -> exactly 3.0.
        let summary = &aggregate(&replicates, 1, 0.5).unwrap()[0];
        assert_eq!(summary.lower, 1.0);
        assert_eq!(summary.upper, 3.0);
    }

    #[test]
    fn aggregate_rejects_singleton_ensemble() {
        let replicates = vec![vec![1.0]];
        assert_eq!(
            aggregate(&replicates, 1, 0.95).unwrap_err(),
            UncertaintyError::InsufficientResamples
        );
    }

    #[test]
    fn replicate_state_depends_only_on_seed_and_index() {
        assert_eq!(replicate_state(7, 3), replicate_state(7, 3));
        assert_ne!(replicate_state(7, 3), replicate_state(7, 4));
        assert_ne!(replicate_state(7, 3), replicate_state(8, 3));
    }

    #[test]
    fn validate_design_flags_dimension_mismatch() {
        let theta = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let target = vec![1.0];
        assert_eq!(
            validate_design(&theta, &target).unwrap_err(),
            UncertaintyError::DimensionMismatch { expected: 2, actual: 1 }
        );
    }
}
