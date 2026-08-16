/// The two primary minimization objectives for an equation candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CandidateMetrics {
    pub mean_squared_error: f64,
    pub complexity: usize,
}

impl CandidateMetrics {
    pub fn dominates(self, other: Self) -> bool {
        (self.mean_squared_error <= other.mean_squared_error && self.complexity <= other.complexity)
            && (self.mean_squared_error < other.mean_squared_error
                || self.complexity < other.complexity)
    }
}
