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
            resource_limits: ResourceLimits::default(),
        }
    }

    /// Enables regime segmentation with default penalty settings. Kept as a
    /// helper so callers (e.g. the CLI) need not depend on `lawsynth-regime`.
    pub fn enable_regimes(&mut self) {
        self.regime = Some(SegmentationConfig::default());
    }
}
