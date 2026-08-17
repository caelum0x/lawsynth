use lawsynth_score::CandidateMetrics;

/// Named, independently scored discovery path retained for comparison.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryBranch {
    pub name: String,
    pub metrics: CandidateMetrics,
    pub source: String,
}
impl DiscoveryBranch {
    pub fn new(
        name: impl Into<String>,
        source: impl Into<String>,
        metrics: CandidateMetrics,
    ) -> Self {
        Self { name: name.into(), source: source.into(), metrics }
    }
}
