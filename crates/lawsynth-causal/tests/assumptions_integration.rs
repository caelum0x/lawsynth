use lawsynth_causal::{AssumptionSet, CausalAssumption, CausalGraph};
#[test]
fn declared_edge_assumption_is_checked_against_graph() {
    let mut graph = CausalGraph::new(["treatment", "outcome"]).unwrap();
    graph.add_edge("treatment", "outcome").unwrap();
    let mut assumptions = AssumptionSet::default();
    assumptions.declare(CausalAssumption::NoUnmeasuredConfounding {
        cause: "treatment".into(),
        effect: "outcome".into(),
    });
    assert!(assumptions.validate_against(&graph).is_ok());
}
