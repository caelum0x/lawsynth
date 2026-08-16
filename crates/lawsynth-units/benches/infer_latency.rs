use std::{hint::black_box, time::Instant};

use lawsynth_core::Identifier;
use lawsynth_expr::parse;
use lawsynth_units::{Unit, infer_expression_dimension};

fn main() {
    let expression = parse("x / t").unwrap();
    let dimensions = [
        (Identifier::new("x").unwrap(), Unit::parse("m").unwrap()),
        (Identifier::new("t").unwrap(), Unit::parse("s").unwrap()),
    ]
    .into_iter()
    .collect();
    let started = Instant::now();
    for _ in 0..100_000 {
        black_box(infer_expression_dimension(&expression, &dimensions).unwrap());
    }
    println!("inferred dimensions in {:?}", started.elapsed());
}
