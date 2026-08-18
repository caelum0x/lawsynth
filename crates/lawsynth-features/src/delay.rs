use crate::FeatureError;

/// A rectangular delayed embedding of one sampled scalar series.
///
/// `start_index` is the first original observation represented by a row.  For
/// example, columns for delays `[0, 2]` over `[a, b, c, d]` start at index 2
/// and contain `[[c, a], [d, b]]`.
#[derive(Clone, Debug, PartialEq)]
pub struct DelayEmbedding {
    pub start_index: usize,
    pub delays: Vec<usize>,
    pub rows: Vec<Vec<f64>>,
}

/// Builds delayed columns without inventing unavailable history.
///
/// The supplied delays are retained in caller order. Every delay must be less
/// than the series length and at least one delay is required; otherwise this
/// returns an error rather than padding with synthetic values.
pub fn delayed_columns(values: &[f64], delays: &[usize]) -> Result<DelayEmbedding, FeatureError> {
    if values.is_empty() {
        return Err(FeatureError::EmptySeries);
    }
    let max_delay = delays.iter().copied().max().ok_or(FeatureError::EmptyVariables)?;
    if max_delay >= values.len() {
        return Err(FeatureError::InvalidDelay { lag: max_delay, length: values.len() });
    }

    let rows = (max_delay..values.len())
        .map(|index| delays.iter().map(|delay| values[index - delay]).collect())
        .collect();
    Ok(DelayEmbedding { start_index: max_delay, delays: delays.to_vec(), rows })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delays_drop_only_the_history_that_does_not_exist() {
        assert_eq!(
            delayed_columns(&[1.0, 2.0, 3.0, 4.0], &[0, 2]).unwrap(),
            DelayEmbedding {
                start_index: 2,
                delays: vec![0, 2],
                rows: vec![vec![3.0, 1.0], vec![4.0, 2.0]],
            }
        );
    }

    #[test]
    fn delay_outside_the_available_history_is_rejected() {
        assert_eq!(
            delayed_columns(&[1.0, 2.0], &[2]),
            Err(FeatureError::InvalidDelay { lag: 2, length: 2 })
        );
    }
}
