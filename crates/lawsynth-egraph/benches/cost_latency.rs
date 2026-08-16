use lawsynth_core::Identifier;
use lawsynth_egraph::expression_cost;
use lawsynth_expr::parse;
use std::{hint::black_box, time::Instant};
fn main() {
    let expression = parse("a*b + c*d + (e+f)^2").unwrap();
    let started = Instant::now();
    let mut cost = 0;
    for _ in 0..1_000_000 {
        cost += black_box(expression_cost(&expression));
    }
    assert_eq!(Identifier::new("x").unwrap().as_str(), "x");
    println!("summed cost {cost} in {:?}", started.elapsed());
}
