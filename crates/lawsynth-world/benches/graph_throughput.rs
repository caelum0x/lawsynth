use lawsynth_expr::parse;
use lawsynth_world::expression_symbols;
use std::{hint::black_box, time::Instant};

fn main() {
    let expression = parse("a * b + c * d + a / (1 + e)").unwrap();
    let started = Instant::now();
    let mut symbols = 0;
    for _ in 0..100_000 {
        symbols += black_box(expression_symbols(&expression)).len();
    }
    assert_eq!(symbols, 500_000);
    println!(
        "walked {symbols} unique symbol references in {:?}",
        started.elapsed()
    );
}
