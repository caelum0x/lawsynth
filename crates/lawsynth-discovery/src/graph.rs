use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_profile::{ProfileError, estimate_delay};

use crate::DiscoveryError;

/// One time-ordered, lagged association hypothesis.
#[derive(Clone, Debug, PartialEq)]
pub struct DependencyEdge {
    pub source: Identifier,
    pub target: Identifier,
    /// Number of samples by which `target` follows `source`.
    pub lag: usize,
    pub correlation: f64,
}

/// Stable set of time-ordered association hypotheses.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DependencyGraph {
    pub edges: Vec<DependencyEdge>,
}

/// Infers lagged association hypotheses from all unordered numeric column pairs.
///
/// An edge is emitted only when one series leads the other by at least one
/// sample and the absolute Pearson correlation reaches `minimum_correlation`.
/// This is an association graph, not a causal-identification claim.
pub fn infer_lagged_dependencies(
    dataset: &Dataset,
    max_lag: usize,
    minimum_correlation: f64,
) -> Result<DependencyGraph, DiscoveryError> {
    if !minimum_correlation.is_finite() || !(0.0..=1.0).contains(&minimum_correlation) {
        return Err(DiscoveryError::Graph(
            "minimum correlation must be finite and between zero and one".to_owned(),
        ));
    }
    let columns = dataset.columns().iter().collect::<Vec<_>>();
    let mut edges = Vec::new();
    for left_index in 0..columns.len() {
        for right_index in left_index + 1..columns.len() {
            let (left_id, left) = columns[left_index];
            let (right_id, right) = columns[right_index];
            let estimate = match estimate_delay(&left.values, &right.values, max_lag) {
                Ok(estimate) => estimate,
                Err(ProfileError::ConstantValues) => continue,
                Err(error) => return Err(DiscoveryError::Profile(error.to_string())),
            };
            if estimate.lag == 0 || estimate.correlation.abs() < minimum_correlation {
                continue;
            }
            let (source, target, lag) = if estimate.lag > 0 {
                (left_id.clone(), right_id.clone(), estimate.lag as usize)
            } else {
                (
                    right_id.clone(),
                    left_id.clone(),
                    estimate.lag.unsigned_abs(),
                )
            };
            edges.push(DependencyEdge {
                source,
                target,
                lag,
                correlation: estimate.correlation,
            });
        }
    }
    Ok(DependencyGraph { edges })
}

#[cfg(test)]
mod tests {
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn infers_a_leading_association_without_claiming_causality() {
        let x = Identifier::new("x").unwrap();
        let y = Identifier::new("y").unwrap();
        let dataset = Dataset::new(
            TimeAxis::new((0..6).map(|value| value as f64).collect()).unwrap(),
            [
                NumericColumn::new(x.clone(), vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]),
                NumericColumn::new(y.clone(), vec![8.0, 0.0, 1.0, 2.0, 3.0, 4.0]),
            ],
        )
        .unwrap();
        let graph = infer_lagged_dependencies(&dataset, 2, 0.99).unwrap();
        assert_eq!(
            graph.edges,
            vec![DependencyEdge {
                source: x,
                target: y,
                lag: 1,
                correlation: 1.0,
            }]
        );
    }
}
