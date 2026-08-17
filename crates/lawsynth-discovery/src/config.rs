use lawsynth_core::{Identifier, ResourceLimits};
use lawsynth_differentiate::DerivativeConfig;
use lawsynth_preprocess::PreprocessPipeline;
use lawsynth_regime::SegmentationConfig;
use lawsynth_sparse::SparseConfig;
use lawsynth_stats::BootstrapConfig;
use lawsynth_symbolic::SymbolicConfig;

/// Sparse solver used for feature-library coefficient fitting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SparseMethod {
    #[default]
    Stlsq,
    Sr3,
}

/// Opt-in joint parameter refinement of a candidate's numeric constants (§8.5).
///
/// After sparse discovery yields a candidate structure, the constants inside its
/// laws are refined against the observed trajectory with a deterministic bounded
/// coordinate search. The budget is fixed here; the pass never reads a wall clock
/// or draws random numbers, so results are fully reproducible.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RefinementConfig {
    /// Hard cap on coordinate-search iterations.
    pub max_iterations: usize,
    /// Initial coordinate step applied uniformly to every refined constant.
    pub initial_step: f64,
    /// Step floor below which the search terminates.
    pub minimum_step: f64,
}

impl Default for RefinementConfig {
    fn default() -> Self {
        Self { max_iterations: 200, initial_step: 0.5, minimum_step: 1e-8 }
    }
}

/// Opt-in dependency and causal hypothesis discovery (§8.6).
///
/// The pass produces a *candidate* causal structure — never a proven-causation
/// claim — from Granger-style predictive direction, a marginal-independence
/// prune, and a time-order gate. Every threshold is fixed here so the hypothesis
/// is deterministic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CausalHypothesisConfig {
    /// Maximum lag considered by the Granger regression.
    pub max_lag: usize,
    /// Minimum sample count required by the Granger regression.
    pub min_samples: usize,
    /// Minimum Granger F statistic for a directed edge to be hypothesized.
    pub minimum_f_statistic: f64,
    /// Marginal absolute correlation at or below which a pair is treated as
    /// independent, so no edge is proposed in either direction.
    pub independence_tolerance: f64,
}

impl Default for CausalHypothesisConfig {
    fn default() -> Self {
        Self { max_lag: 1, min_samples: 12, minimum_f_statistic: 4.0, independence_tolerance: 0.05 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryConfig {
    pub state: Vec<Identifier>,
    pub polynomial_degree: usize,
    pub include_trigonometric: bool,
    pub include_rational: bool,
    pub symbolic: Option<SymbolicConfig>,
    pub sparse: SparseConfig,
    pub sparse_method: SparseMethod,
    pub derivative: DerivativeConfig,
    pub smoothing_radius: Option<usize>,
    pub preprocessing: Option<PreprocessPipeline>,
    pub bootstrap: Option<BootstrapConfig>,
    /// Opt-in regime segmentation of the primary state window. Default `None`
    /// keeps the fast path untouched; enable via [`DiscoveryConfig::enable_regimes`].
    pub regime: Option<SegmentationConfig>,
    /// Opt-in joint parameter refinement (§8.5). Default `None` leaves the fast
    /// path and its metrics untouched; enable via
    /// [`DiscoveryConfig::enable_refinement`].
    pub refine: Option<RefinementConfig>,
    /// Opt-in dependency/causal hypothesis discovery (§8.6). Default `None`
    /// produces no hypothesis; enable via
    /// [`DiscoveryConfig::enable_causal_hypothesis`].
    pub causal: Option<CausalHypothesisConfig>,
    /// Hard bounds enforced before data profiling and feature expansion.
    pub resource_limits: ResourceLimits,
}

impl DiscoveryConfig {
    pub fn new(state: impl IntoIterator<Item = Identifier>) -> Self {
        Self {
            state: state.into_iter().collect(),
            polynomial_degree: 2,
            include_trigonometric: false,
            include_rational: false,
            symbolic: None,
            sparse: SparseConfig::default(),
            sparse_method: SparseMethod::default(),
            derivative: DerivativeConfig::default(),
            smoothing_radius: None,
            preprocessing: None,
            bootstrap: None,
            regime: None,
            refine: None,
            causal: None,
            resource_limits: ResourceLimits::default(),
        }
    }

    /// Enables regime segmentation with default penalty settings. Kept as a
    /// helper so callers (e.g. the CLI) need not depend on `lawsynth-regime`.
    pub fn enable_regimes(&mut self) {
        self.regime = Some(SegmentationConfig::default());
    }

    /// Enables joint parameter refinement (§8.5) with default budget settings.
    pub fn enable_refinement(&mut self) {
        self.refine = Some(RefinementConfig::default());
    }

    /// Enables dependency/causal hypothesis discovery (§8.6) with default
    /// thresholds.
    pub fn enable_causal_hypothesis(&mut self) {
        self.causal = Some(CausalHypothesisConfig::default());
    }
}
