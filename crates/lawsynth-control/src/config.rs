use lawsynth_differentiate::DerivativeConfig;
use lawsynth_features::FeatureConfig;
use lawsynth_sparse::SparseConfig;

/// Deterministic configuration for a controlled (SINDYc) discovery run.
///
/// Each field reuses the configuration type of the crate it drives, so the
/// behaviour of the augmented library, the derivative estimator, and the sparse
/// solve match those subsystems exactly. All defaults are fixed constants — no
/// wall clock, no environment — so a default run is fully reproducible.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ControlConfig {
    /// Polynomial degree and constant-term policy for the augmented library
    /// `Θ(x, u)` built over the combined `[states.., controls..]` variables.
    pub features: FeatureConfig,
    /// Derivative estimator used to form the state-derivative targets `ẋ`.
    /// Controls are never passed to this estimator.
    pub derivative: DerivativeConfig,
    /// Sequentially-thresholded least-squares configuration for the per-state
    /// sparse regression `Θ(x, u) ξ ≈ ẋ`.
    pub sparse: SparseConfig,
}
