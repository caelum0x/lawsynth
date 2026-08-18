use lawsynth_core::Identifier;
use lawsynth_differentiate::DerivativeMethod;

use crate::ImplicitError;

/// Configuration for [`implicit_discover`](crate::implicit_discover).
///
/// Every field is explicit and deterministic; there is no hidden randomness,
/// tolerance auto-tuning, or wall-clock dependence.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitConfig {
    /// Maximum total polynomial degree of the state monomials that seed the
    /// augmented library `Θ(x, ẋ)`. Must be at least 1.
    pub degree: usize,
    /// Whether to include the pure-state constant term `1` in the library. The
    /// bare derivative term `ẋ` is always present regardless of this flag.
    pub include_constant: bool,
    /// Sequentially-thresholded least-squares cutoff, applied in the
    /// RMS-standardised feature space of each alternating-LHS regression.
    pub threshold: f64,
    /// Ridge regularisation added to every normal-equation diagonal. A small
    /// positive value keeps collinear augmented columns numerically stable.
    pub ridge: f64,
    /// Maximum STLSQ pruning iterations per candidate left-hand side.
    pub max_iterations: usize,
    /// Weight on the sparsity term in the candidate-selection score
    /// (`relative_residual + sparsity_weight · active/library`).
    pub sparsity_weight: f64,
    /// Relative-residual ceiling below which a relation is deemed *consistent*
    /// and its explicit rational law is reconstructed.
    pub consistency_tolerance: f64,
    /// Smallest `|Q(x)|` over the samples below which the reconstructed
    /// denominator is flagged as approaching a pole.
    pub min_denominator: f64,
    /// Derivative estimation method for the target state.
    pub derivative: DerivativeMethod,
    /// Drop the first and last sample, whose one-sided derivative estimates are
    /// only first-order accurate, before assembling the library.
    pub trim_boundary: bool,
    /// Target state whose dynamics to discover. When `None`, the first
    /// identifier-sorted dataset column is used.
    pub target: Option<Identifier>,
}

impl Default for ImplicitConfig {
    fn default() -> Self {
        Self {
            degree: 2,
            include_constant: true,
            threshold: 0.05,
            ridge: 1e-8,
            max_iterations: 20,
            sparsity_weight: 0.05,
            consistency_tolerance: 1e-2,
            min_denominator: 1e-6,
            derivative: DerivativeMethod::FiniteDifference,
            trim_boundary: true,
            target: None,
        }
    }
}

impl ImplicitConfig {
    pub(crate) fn validate(&self) -> Result<(), ImplicitError> {
        let finite_non_negative = |value: f64| value.is_finite() && value >= 0.0;
        if self.degree == 0
            || self.max_iterations == 0
            || !finite_non_negative(self.threshold)
            || !finite_non_negative(self.ridge)
            || !finite_non_negative(self.sparsity_weight)
            || !finite_non_negative(self.consistency_tolerance)
            || !finite_non_negative(self.min_denominator)
        {
            return Err(ImplicitError::InvalidConfig);
        }
        Ok(())
    }
}
