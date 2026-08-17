use lawsynth_preprocess::AppliedTransform;
use lawsynth_profile::DatasetProfile;
use lawsynth_regime::Segmentation;
use lawsynth_score::CandidateMetrics;
use lawsynth_stats::PercentileInterval;
use lawsynth_world::World;

use crate::pareto::{CandidateScore, pareto_frontier};

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
