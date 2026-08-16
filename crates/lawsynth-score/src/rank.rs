use crate::{CandidateMetrics, ScoreError, ScoringConfig};

/// Orders candidates by error, then complexity, then their original index.
///
/// The index tie-break makes equal metrics reproducible without pretending the
/// candidates have a meaningful numerical distinction.
pub fn rank_candidates(metrics: &[CandidateMetrics]) -> Vec<usize> {
    let mut indices = (0..metrics.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| {
        metrics[*left]
            .mean_squared_error
            .total_cmp(&metrics[*right].mean_squared_error)
            .then_with(|| metrics[*left].complexity.cmp(&metrics[*right].complexity))
            .then_with(|| left.cmp(right))
    });
    indices
}

/// Returns deterministic weighted objective values for scalar ranking.
pub fn weighted_rank(
    metrics: &[CandidateMetrics],
    config: ScoringConfig,
) -> Result<Vec<(usize, f64)>, ScoreError> {
    if !config.error_weight.is_finite()
        || config.error_weight < 0.0
        || !config.complexity_weight.is_finite()
        || config.complexity_weight < 0.0
    {
        return Err(ScoreError::InvalidConfig);
    }
    let mut values = metrics
        .iter()
        .enumerate()
        .map(|(index, metric)| {
            (
                index,
                config.error_weight * metric.mean_squared_error
                    + config.complexity_weight * metric.complexity as f64,
            )
        })
        .collect::<Vec<_>>();
    if values.iter().any(|(_, value)| !value.is_finite()) {
        return Err(ScoreError::NonFiniteValue);
    }
    values.sort_by(|(left_index, left), (right_index, right)| {
        left.total_cmp(right)
            .then_with(|| left_index.cmp(right_index))
    });
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_is_stable_for_exact_ties() {
        let metrics = [
            CandidateMetrics {
                mean_squared_error: 1.0,
                complexity: 2,
            },
            CandidateMetrics {
                mean_squared_error: 1.0,
                complexity: 2,
            },
            CandidateMetrics {
                mean_squared_error: 0.5,
                complexity: 3,
            },
        ];
        assert_eq!(rank_candidates(&metrics), vec![2, 0, 1]);
        assert_eq!(
            weighted_rank(&metrics, ScoringConfig::default()).unwrap()[0].0,
            2
        );
    }
}
