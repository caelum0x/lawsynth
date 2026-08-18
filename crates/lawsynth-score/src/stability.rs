use crate::ScoreError;

/// Agreement diagnostics for a collection of boolean feature selections.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionStability {
    pub selection_frequencies: Vec<f64>,
    pub mean_pairwise_jaccard: f64,
}

/// Calculates selection frequencies and mean pairwise Jaccard similarity.
///
/// Two entirely empty selections agree perfectly (Jaccard 1.0), because they
/// make the same structural decision and no arbitrary zero-denominator choice
/// is allowed to make that agreement look unstable.
pub fn selection_stability(selections: &[Vec<bool>]) -> Result<SelectionStability, ScoreError> {
    let Some(first) = selections.first() else {
        return Err(ScoreError::EmptyObservations);
    };
    if selections.iter().any(|selection| selection.len() != first.len()) {
        return Err(ScoreError::InconsistentSelectionWidth);
    }
    let selection_frequencies = (0..first.len())
        .map(|feature| {
            selections.iter().filter(|selection| selection[feature]).count() as f64
                / selections.len() as f64
        })
        .collect();
    let mut total = 0.0;
    let mut pairs = 0usize;
    for left in 0..selections.len() {
        for right in left + 1..selections.len() {
            let (intersection, union) = selections[left].iter().zip(&selections[right]).fold(
                (0usize, 0usize),
                |(intersection, union), (left, right)| {
                    (
                        intersection + usize::from(*left && *right),
                        union + usize::from(*left || *right),
                    )
                },
            );
            total += if union == 0 { 1.0 } else { intersection as f64 / union as f64 };
            pairs += 1;
        }
    }
    Ok(SelectionStability {
        selection_frequencies,
        mean_pairwise_jaccard: if pairs == 0 { 1.0 } else { total / pairs as f64 },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_frequency_and_pairwise_selection_agreement() {
        let value = selection_stability(&[vec![true, false], vec![true, true]]).unwrap();
        assert_eq!(value.selection_frequencies, vec![1.0, 0.5]);
        assert!((value.mean_pairwise_jaccard - 0.5).abs() < 1e-12);
    }
}
