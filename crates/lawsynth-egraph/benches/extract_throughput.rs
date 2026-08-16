use lawsynth_egraph::{EquivalenceGraph, RewriteConfig, expression_cost, extract_lowest_cost};
use lawsynth_expr::parse;
use std::{hint::black_box, time::Instant};
fn main() {
    let mut graph = EquivalenceGraph::default();
    graph.add(parse("0 + (x * 1)").unwrap(), &RewriteConfig::default());
    let members = graph.classes().next().unwrap().members.clone();
    let started = Instant::now();
    let mut nodes = 0;
    for _ in 0..100_000 {
        nodes += black_box(expression_cost(&extract_lowest_cost(&members).unwrap()));
    }
    println!("extracted {nodes} nodes in {:?}", started.elapsed());
}
