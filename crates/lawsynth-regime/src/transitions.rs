use crate::{RegimeError, Result};
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionMatrix {
    pub probabilities: Vec<Vec<f64>>,
    pub counts: Vec<Vec<usize>>,
}
impl TransitionMatrix {
    pub fn from_states(states: &[usize], state_count: usize) -> Result<Self> {
        if state_count == 0 {
            return Err(RegimeError::InvalidParameter("state_count"));
        }
        if states.iter().any(|&s| s >= state_count) {
            return Err(RegimeError::InvalidParameter("state"));
        }
        let mut counts = vec![vec![0; state_count]; state_count];
        for pair in states.windows(2) {
            counts[pair[0]][pair[1]] += 1;
        }
        let probabilities = counts
            .iter()
            .map(|row| {
                let total: usize = row.iter().sum();
                if total == 0 {
                    vec![0.0; state_count]
                } else {
                    row.iter().map(|&v| v as f64 / total as f64).collect()
                }
            })
            .collect();
        Ok(Self { probabilities, counts })
    }
    pub fn probability(&self, from: usize, to: usize) -> Option<f64> {
        self.probabilities.get(from)?.get(to).copied()
    }
}
