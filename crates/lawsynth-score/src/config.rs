/// Weights for deterministic scalar ranking after Pareto filtering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoringConfig {
    pub error_weight: f64,
    pub complexity_weight: f64,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self { error_weight: 1.0, complexity_weight: 0.01 }
    }
}
