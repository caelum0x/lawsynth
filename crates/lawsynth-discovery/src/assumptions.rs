use std::collections::BTreeSet;

use lawsynth_core::Identifier;

use crate::{DependencyGraph, DiscoveryError};

/// A directed structural constraint expressed in World-IR identifiers.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct EdgeConstraint {
    pub source: Identifier,
    pub target: Identifier,
}

impl EdgeConstraint {
    pub fn new(source: Identifier, target: Identifier) -> Self {
        Self { source, target }
    }
}

/// Required and forbidden association hypotheses supplied by a domain expert.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DependencyAssumptions {
    pub required: BTreeSet<EdgeConstraint>,
    pub forbidden: BTreeSet<EdgeConstraint>,
}

impl DependencyAssumptions {
    /// Applies constraints to a graph and rejects incompatible or unmet claims.
    pub fn apply(&self, graph: &DependencyGraph) -> Result<DependencyGraph, DiscoveryError> {
        if self.required.iter().any(|edge| self.forbidden.contains(edge)) {
            return Err(DiscoveryError::Graph(
                "an edge cannot be both required and forbidden".to_owned(),
            ));
        }
        let edges = graph
            .edges
            .iter()
            .filter(|edge| {
                !self
                    .forbidden
                    .contains(&EdgeConstraint::new(edge.source.clone(), edge.target.clone()))
            })
            .cloned()
            .collect::<Vec<_>>();
        for required in &self.required {
            if !edges
                .iter()
                .any(|edge| edge.source == required.source && edge.target == required.target)
            {
                return Err(DiscoveryError::Graph(format!(
                    "required edge '{} -> {}' was not inferred",
                    required.source, required.target
                )));
            }
        }
        Ok(DependencyGraph { edges })
    }
}

#[cfg(test)]
mod tests {
    use crate::DependencyEdge;

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn filters_forbidden_edges_and_requires_evidence() {
        let xy = EdgeConstraint::new(id("x"), id("y"));
        let yz = EdgeConstraint::new(id("y"), id("z"));
        let graph = DependencyGraph {
            edges: vec![
                DependencyEdge { source: id("x"), target: id("y"), lag: 1, correlation: 0.9 },
                DependencyEdge { source: id("y"), target: id("z"), lag: 1, correlation: 0.8 },
            ],
        };
        let assumptions = DependencyAssumptions {
            required: BTreeSet::from([xy]),
            forbidden: BTreeSet::from([yz]),
        };
        assert_eq!(assumptions.apply(&graph).unwrap().edges.len(), 1);
    }
}
