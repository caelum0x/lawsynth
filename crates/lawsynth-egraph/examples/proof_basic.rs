use lawsynth_core::Identifier;
use lawsynth_egraph::{EquivalenceGraph, RewriteConfig};
use lawsynth_expr::Expr;
fn main() {
    let expression = Expr::sum(
        Expr::constant(0.0),
        Expr::symbol(Identifier::new("x").unwrap()),
    );
    let mut graph = EquivalenceGraph::default();
    graph.add(expression, &RewriteConfig::default());
    println!("equivalence classes: {}", graph.classes().count());
}
