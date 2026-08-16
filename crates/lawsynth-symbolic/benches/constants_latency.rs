use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_symbolic::calibrate_affine;
use std::{hint::black_box, time::Instant};
fn main() {
    let x = Identifier::new("x").unwrap();
    let expression = Expr::symbol(x.clone());
    let contexts = (0..1_000)
        .map(|i| [(x.clone(), i as f64)].into_iter().collect())
        .collect::<Vec<_>>();
    let targets = (0..1_000).map(|i| 2.0 * i as f64 + 1.0).collect::<Vec<_>>();
    let started = Instant::now();
    let mut count = 0;
    for _ in 0..100 {
        let calibrated = calibrate_affine(&expression, &contexts, &targets).unwrap();
        count += black_box(calibrated.expression.to_canonical_string().len());
    }
    println!("calibrated {count} nodes in {:?}", started.elapsed());
}
