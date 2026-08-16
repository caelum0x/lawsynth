use lawsynth_score::information_criteria;
use std::{hint::black_box, time::Instant};

fn main() {
    let started = Instant::now();
    let mut aic = 0.0;
    for _ in 0..1_000_000 {
        aic += black_box(information_criteria(8, 500, 100.0).unwrap()).aic;
    }
    println!("computed AIC sum {aic:.1} in {:?}", started.elapsed());
}
