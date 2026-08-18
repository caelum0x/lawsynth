use lawsynth_causal::{CausalAssumption, CausalGraph};
use lawsynth_preprocess::AppliedTransform;
use lawsynth_profile::DatasetProfile;
use lawsynth_regime::Segmentation;
use lawsynth_score::CandidateMetrics;
use lawsynth_stats::PercentileInterval;
use lawsynth_world::World;

use crate::pareto::{CandidateScore, pareto_frontier};

/// Outcome of the opt-in joint parameter refinement pass (§8.5).
///
/// The mean-squared errors are measured over the *simulated trajectory* (the
/// candidate integrated forward from the first observation), which is the
/// quantity the refinement optimizes. Because the optimizer starts from the
/// discovered constants and only accepts improvements, `mse_after` never exceeds
/// `mse_before`.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterRefinement {
    /// Refined constants in the candidate's deterministic pre-order.
    pub parameters: Vec<f64>,
    /// Trajectory mean-squared error at the discovered (pre-refinement) constants.
    pub mse_before: f64,
    /// Trajectory mean-squared error at the refined constants.
    pub mse_after: f64,
    /// Coordinate-search iterations actually consumed.
    pub iterations: usize,
}

impl ParameterRefinement {
    /// Non-negative fit improvement `mse_before - mse_after`.
    pub fn improvement(&self) -> f64 {
        self.mse_before - self.mse_after
    }
}

/// Tally of the in-loop dimensional pruning pass (`specs/dimensional-search/`).
///
/// Present only when units are supplied in the
/// [`DiscoveryConfig`](crate::DiscoveryConfig); `None` on the default path.
/// `considered` counts every `(candidate term, target state)` admissibility test
/// performed, `pruned` the subset rejected as dimensionally inconsistent. The
/// tally is diagnostic only — it never affects which world is returned beyond the
/// removal of the pruned terms.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DimensionalPruningReport {
    /// Total admissibility tests performed across states and search paths.
    pub considered: usize,
    /// Candidate terms rejected as dimensionally inconsistent.
    pub pruned: usize,
}

impl DimensionalPruningReport {
    /// Records one admissibility test outcome (`pruned` when rejected).
    pub(crate) fn record(&mut self, pruned: bool) {
        self.considered += 1;
        if pruned {
            self.pruned += 1;
        }
    }

    /// Candidate terms retained after pruning.
    pub fn retained(&self) -> usize {
        self.considered - self.pruned
    }
}

/// An executable equation system fitted from one discovery branch.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryCandidate {
    pub world: World,
    pub metrics: CandidateMetrics,
    pub bootstrap_mse: Option<PercentileInterval>,
    /// Bootstrap selection-stability summary in `[0, 1]`, higher is more stable.
    ///
    /// Populated only when bootstrap uncertainty is enabled in the
    /// [`DiscoveryConfig`](crate::DiscoveryConfig); `None` on the default path.
    pub stability: Option<f64>,
    /// Joint parameter-refinement outcome (§8.5). Populated only when refinement
    /// is enabled in the [`DiscoveryConfig`](crate::DiscoveryConfig); `None` on
    /// the default path. When refinement strictly improves the trajectory fit,
    /// [`world`](Self::world) already carries the refined constants.
    pub refinement: Option<ParameterRefinement>,
}

impl DiscoveryCandidate {
    /// Projects the candidate onto the §16 multi-objective score vector used for
    /// Pareto comparison: error and complexity are minimized, stability is
    /// maximized. Absent stability is treated as the least-stable value so that
    /// candidates without bootstrap evidence never dominate on that axis.
    pub fn score(&self) -> CandidateScore {
        CandidateScore {
            error: self.metrics.mean_squared_error,
            complexity: self.metrics.complexity,
            stability: self.stability.unwrap_or(0.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryResult {
    pub profile: DatasetProfile,
    pub preprocessing: Vec<AppliedTransform>,
    pub candidates: Vec<DiscoveryCandidate>,
    /// Indices into [`candidates`](Self::candidates) that form the non-dominated
    /// Pareto frontier over the multi-objective score vector (§8, §16).
    pub frontier: Vec<usize>,
    /// Regime segmentation of the primary state, present only when the opt-in
    /// regime pass is enabled in the [`DiscoveryConfig`](crate::DiscoveryConfig).
    pub regimes: Option<Segmentation>,
    /// Candidate dependency/causal structure (§8.6), present only when the
    /// opt-in causal pass is enabled in the
    /// [`DiscoveryConfig`](crate::DiscoveryConfig).
    ///
    /// This is a **hypothesis**, not a proven-causation claim: edges are
    /// Granger-style predictive directions retained under a time-order gate and a
    /// marginal-independence prune. The assumptions the hypothesis relies on are
    /// reported alongside in [`dependency_assumptions`](Self::dependency_assumptions).
    pub dependency_hypothesis: Option<CausalGraph>,
    /// The causal assumptions under which [`dependency_hypothesis`](Self::dependency_hypothesis)
    /// would license a causal reading (e.g. faithfulness, causal sufficiency).
    /// Set together with the hypothesis; `None` on the default path.
    pub dependency_assumptions: Option<Vec<CausalAssumption>>,
    /// Tally of in-loop dimensional pruning, present only when units are supplied
    /// in the [`DiscoveryConfig`](crate::DiscoveryConfig); `None` on the default path.
    pub dimensional_pruning: Option<DimensionalPruningReport>,
    /// Auditable drop report for the grammar template prior
    /// (`specs/template-priors/`), present only when a
    /// [`TemplatePrior`](crate::TemplatePrior) is supplied in the
    /// [`DiscoveryConfig`](crate::DiscoveryConfig); `None` on the default path.
    /// Records every candidate term the prior dropped and why.
    pub template_filter: Option<crate::TemplateFilterReport>,
}

impl DiscoveryResult {
    /// Returns references to the candidates on the Pareto frontier, in the order
    /// they appear in [`candidates`](Self::candidates).
    pub fn pareto_frontier(&self) -> Vec<&DiscoveryCandidate> {
        self.frontier.iter().filter_map(|index| self.candidates.get(*index)).collect()
    }

    /// Recomputes the frontier indices for a candidate set. Exposed so callers
    /// constructing results directly stay consistent with [`pareto_frontier`].
    pub(crate) fn compute_frontier(candidates: &[DiscoveryCandidate]) -> Vec<usize> {
        pareto_frontier(candidates)
    }
}
