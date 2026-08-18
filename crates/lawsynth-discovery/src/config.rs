use std::collections::BTreeMap;

use lawsynth_core::{Identifier, ResourceLimits};
use lawsynth_differentiate::DerivativeConfig;
use lawsynth_preprocess::PreprocessPipeline;
use lawsynth_regime::SegmentationConfig;
use lawsynth_sparse::SparseConfig;
use lawsynth_stats::BootstrapConfig;
use lawsynth_symbolic::SymbolicConfig;
use lawsynth_units::Dimension;

use crate::TemplatePrior;

/// Sparse solver used for feature-library coefficient fitting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SparseMethod {
    #[default]
    Stlsq,
    Sr3,
    /// Forward Regression with Orthogonal Least Squares: greedy error-reduction
    /// ratio (ERR) selection with a Gram-Schmidt orthogonalisation, deterministic.
    Frols,
    /// Stepwise Sparse Regression: prune the full fit one term at a time and pick
    /// the support minimising the Akaike information criterion, deterministic.
    Ssr,
    /// Stability-biased SR3 ("trapping"): damps a positive linear self-feedback
    /// term toward the bounded regime. A stability *bias*, not a boundedness proof.
    Trapping,
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

/// Opt-in per-variable units enabling dimensional pruning of candidate terms
/// (`specs/dimensional-search/`).
///
/// When present, discovery rejects any candidate term whose inferred dimension is
/// impossible or cannot be rescaled to the target derivative's dimension *before*
/// scoring, mirroring PySR/AI-Feynman in-loop dimensional analysis. The map holds
/// only the variables whose unit is known; an undeclared variable is a dimensional
/// wildcard, so a partially-annotated dataset never over-prunes. Absent entirely
/// (`DiscoveryConfig::units == None`), discovery is byte-identical to before.
#[derive(Clone, Debug, PartialEq)]
pub struct DimensionalUnits {
    /// Known SI dimension of each annotated variable.
    dimensions: BTreeMap<Identifier, Dimension>,
    /// Dimension of the time axis the derivatives are taken against. Defaults to
    /// [`Dimension::TIME`]; only the *dimension* matters, never the unit's scale.
    time: Dimension,
}

impl DimensionalUnits {
    /// Builds a unit annotation from `(variable, dimension)` pairs, taking the
    /// time axis to be a duration ([`Dimension::TIME`]).
    pub fn new(dimensions: impl IntoIterator<Item = (Identifier, Dimension)>) -> Self {
        Self { dimensions: dimensions.into_iter().collect(), time: Dimension::TIME }
    }

    /// Overrides the time-axis dimension (e.g. for a dimensionless index axis).
    pub fn with_time_dimension(mut self, time: Dimension) -> Self {
        self.time = time;
        self
    }

    /// The known per-variable dimensions, used by the wildcard-aware inference.
    pub fn dimensions(&self) -> &BTreeMap<Identifier, Dimension> {
        &self.dimensions
    }

    /// The target dimension of `d(state)/dt`, i.e. `[state] / [time]`, or `None`
    /// when the state variable carries no declared unit (pruning is then skipped
    /// for that state).
    pub fn target_dimension(&self, state: &Identifier) -> Option<Dimension> {
        self.dimensions.get(state).and_then(|state| state.divide(self.time))
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
    /// Opt-in per-variable units enabling in-loop dimensional pruning
    /// (`specs/dimensional-search/`). Default `None` leaves the fast path and its
    /// results byte-identical; enable via [`DiscoveryConfig::enable_units`].
    pub units: Option<DimensionalUnits>,
    /// Opt-in grammar-constrained candidate library (`specs/template-priors/`).
    /// A [`TemplatePrior`] is a deterministic hard filter over candidate terms,
    /// applied to the materialised feature library before the sparse solve.
    /// Default `None` admits every candidate term, leaving discovery
    /// byte-identical; enable via [`DiscoveryConfig::with_template_prior`].
    pub template_prior: Option<TemplatePrior>,
    /// Two-sided confidence level for the per-coefficient bootstrap intervals
    /// (`crates/lawsynth-uncertainty`). Only consulted when [`bootstrap`](Self::bootstrap)
    /// is `Some`, in which case discovery also attaches a per-state
    /// [`StateCoefficientEnsemble`](crate::StateCoefficientEnsemble). Defaults to
    /// `0.95`; it never affects the default (no-bootstrap) path.
    pub coefficient_confidence: f64,
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
            units: None,
            template_prior: None,
            coefficient_confidence: 0.95,
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

    /// Enables in-loop dimensional pruning with the supplied per-variable units
    /// (`specs/dimensional-search/`). Candidate terms inconsistent with the target
    /// derivative's dimension are rejected before scoring.
    pub fn enable_units(&mut self, units: DimensionalUnits) {
        self.units = Some(units);
    }

    /// Constrains discovery with a grammar template prior
    /// (`specs/template-priors/`). The prior is a deterministic hard filter over
    /// candidate terms, applied to the feature library before the sparse solve.
    pub fn with_template_prior(&mut self, prior: TemplatePrior) {
        self.template_prior = Some(prior);
    }
}
