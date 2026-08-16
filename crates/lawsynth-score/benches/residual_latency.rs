use lawsynth_score::residuals;
use std::{hint::black_box, time::Instant};

fn main() {
    let observed = (0..20_000).map(|i| i as f64).collect::<Vec<_>>();
    let predicted = observed.iter().map(|v| v + 0.1).collect::<Vec<_>>();
    let started = Instant::now();
    let mut values = 0;
    for _ in 0..500 {
        values += black_box(residuals(&observed, &predicted).unwrap()).len();
    }
    println!("formed {values} residuals in {:?}", started.elapsed());
}
