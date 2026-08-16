use std::{hint::black_box, time::Instant};

use lawsynth_expr::parse;

fn main() {
    let source = "sin(x)^2 + cos(x)^2 + 2.5 * y / (1 + z)";
    let started = Instant::now();
    let mut nodes = 0;
    for _ in 0..20_000 {
        nodes += black_box(parse(source).expect("valid source").to_canonical_string()).len();
    }
    println!("parsed {nodes} AST nodes in {:?}", started.elapsed());
}
