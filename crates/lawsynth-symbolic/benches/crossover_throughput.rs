use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_symbolic::crossover_sum;
use std::{hint::black_box, time::Instant};
fn main() {
    let x = Expr::symbol(Identifier::new("x").unwrap());
    let y = Expr::symbol(Identifier::new("y").unwrap());
    let started = Instant::now();
    let mut nodes = 0;
    for _ in 0..1_000_000 {
        nodes += black_box(crossover_sum(&x, &y).to_canonical_string()).len();
    }
    println!("created {nodes} crossover nodes in {:?}", started.elapsed());
}
