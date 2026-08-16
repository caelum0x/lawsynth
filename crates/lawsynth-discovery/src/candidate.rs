use lawsynth_preprocess::AppliedTransform;
use lawsynth_profile::DatasetProfile;
use lawsynth_score::CandidateMetrics;
use lawsynth_stats::PercentileInterval;
use lawsynth_world::World;

/// An executable equation system fitted from one discovery branch.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryCandidate {
    pub world: World,
    pub metrics: CandidateMetrics,
    pub bootstrap_mse: Option<PercentileInterval>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryResult {
    pub profile: DatasetProfile,
    pub preprocessing: Vec<AppliedTransform>,
    pub candidates: Vec<DiscoveryCandidate>,
}
