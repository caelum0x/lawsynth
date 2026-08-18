use lawsynth_differentiate::DerivativeConfig;
use lawsynth_features::FeatureConfig;
use lawsynth_sparse::SparseConfig;

/// Deterministic configuration for a coupling-structure discovery run.
///
/// Every field reuses the configuration type of the crate it drives, so the
/// behaviour of the per-node candidate library, the derivative estimator, and
/// the sparse solve match those subsystems exactly. All defaults are fixed
/// constants — no wall clock, no environment — so a default run is fully
/// reproducible.
///
/// # The two thresholds
///
/// A network run has two independent thresholds with distinct jobs:
///
/// - [`sparse.threshold`](SparseConfig::threshold) decides which *library terms*
///   survive the per-node regression (a sparsity knob in standardized space).
/// - [`edge_threshold`](Self::edge_threshold) decides which *cross couplings* are
///   promoted to a boolean adjacency edge. A node `j` is only reported as a
///   driver of node `i` when the aggregated magnitude of the surviving terms
///   involving `x_j` reaches this bound. Raising it suppresses weak couplings
///   (fewer false positives, lower sensitivity); lowering it recovers weaker
///   edges at the risk of confounded ones.
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkConfig {
    /// Polynomial degree and constant-term policy for the per-node candidate
    /// library built over **all** node states `{x_1 .. x_N}`.
    ///
    /// Degree 1 recovers linear couplings; degree ≥ 2 adds quadratic and
    /// interaction terms so nonlinear and product couplings can enter the
    /// regression, at the cost of more collinearity between candidate columns.
    pub features: FeatureConfig,
    /// Derivative estimator used to form each node's target `ẋ_i`.
    pub derivative: DerivativeConfig,
    /// Sequentially-thresholded least-squares configuration for the per-node
    /// sparse regression `ẋ_i ≈ Θ ξ_i`.
    pub sparse: SparseConfig,
    /// Minimum aggregated edge strength for a cross coupling to be reported as a
    /// boolean adjacency edge. MUST be finite and non-negative.
    pub edge_threshold: f64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            // Linear coupling is the safe default for structure discovery: it
            // keeps the candidate columns well separated and avoids the extra
            // confounding that quadratic/interaction terms introduce. Callers
            // recovering nonlinear couplings raise the degree explicitly.
            features: FeatureConfig { polynomial_degree: 1, include_constant: true },
            derivative: DerivativeConfig::default(),
            sparse: SparseConfig::default(),
            edge_threshold: 0.05,
        }
    }
}

impl NetworkConfig {
    pub(crate) fn validate(&self) -> Result<(), crate::NetworkError> {
        if !self.edge_threshold.is_finite() || self.edge_threshold < 0.0 {
            return Err(crate::NetworkError::InvalidThreshold(self.edge_threshold));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_linear_and_reproducible() {
        let config = NetworkConfig::default();
        assert_eq!(config.features.polynomial_degree, 1);
        assert!(config.features.include_constant);
        assert_eq!(config.edge_threshold, 0.05);
        assert_eq!(config, NetworkConfig::default());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn rejects_non_finite_or_negative_threshold() {
        let mut config = NetworkConfig { edge_threshold: -1.0, ..NetworkConfig::default() };
        assert!(config.validate().is_err());
        config.edge_threshold = f64::NAN;
        assert!(config.validate().is_err());
    }
}
