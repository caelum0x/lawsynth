use lawsynth_causal::{CausalError, CausalGraph};
#[test]
fn graph_rejects_cycles_and_orders_nodes() {
    let mut graph = CausalGraph::new(["rain", "soil", "growth"]).unwrap();
    graph.add_edge("rain", "soil").unwrap();
    graph.add_edge("soil", "growth").unwrap();
    assert_eq!(graph.topological_order(), vec!["rain", "soil", "growth"]);
    assert!(matches!(graph.add_edge("growth", "rain"), Err(CausalError::Cycle { .. })));
}
