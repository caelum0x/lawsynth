use crate::CandidateMetrics;

/// Returns indices of non-dominated candidates, preserving input order.
pub fn pareto_front(metrics: &[CandidateMetrics]) -> Vec<usize> {
    metrics
        .iter()
        .enumerate()
        .filter_map(|(index, metric)| {
            (!metrics
                .iter()
                .enumerate()
                .any(|(other_index, other)| other_index != index && other.dominates(*metric)))
            .then_some(index)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_dominated_candidates() {
        let front = pareto_front(&[
            CandidateMetrics {
                mean_squared_error: 1.0,
                complexity: 1,
            },
            CandidateMetrics {
                mean_squared_error: 2.0,
                complexity: 2,
            },
            CandidateMetrics {
                mean_squared_error: 0.5,
                complexity: 3,
            },
        ]);
        assert_eq!(front, vec![0, 2]);
    }
}
