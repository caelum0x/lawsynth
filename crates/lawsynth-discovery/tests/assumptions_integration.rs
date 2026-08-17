use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_discovery::{
    DependencyAssumptions, DependencyEdge, DependencyGraph, DiscoveryError, EdgeConstraint,
};

#[test]
fn contradictory_required_and_forbidden_constraints_are_rejected_before_graph_filtering() {
    let x = Identifier::new("x").unwrap();
    let y = Identifier::new("y").unwrap();
    let edge = EdgeConstraint::new(x.clone(), y.clone());
    let graph = DependencyGraph {
        edges: vec![DependencyEdge { source: x, target: y, lag: 1, correlation: 1.0 }],
    };
    let assumptions = DependencyAssumptions {
        required: BTreeSet::from([edge.clone()]),
        forbidden: BTreeSet::from([edge]),
    };
    assert!(
        matches!(assumptions.apply(&graph), Err(DiscoveryError::Graph(message)) if message.contains("both required and forbidden"))
    );
}
