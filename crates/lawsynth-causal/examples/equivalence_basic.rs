use lawsynth_causal::{CausalGraph, equivalence_class};
fn main() {
    let mut graph = CausalGraph::new(["rain", "soil", "yield"]).expect("variables");
    graph.add_edge("rain", "soil").expect("acyclic");
    graph.add_edge("soil", "yield").expect("acyclic");
    let class = equivalence_class(&graph);
    println!(
        "{} undirected adjacencies, {} unshielded colliders",
        class.skeleton.len(),
        class.colliders.len()
    );
}
