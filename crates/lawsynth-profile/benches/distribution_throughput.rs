use lawsynth_profile::distribution;
use std::{hint::black_box, time::Instant};

fn main() {
    let values = (0..20_000)
        .map(|i| ((i as f64) * 0.001).sin())
        .collect::<Vec<_>>();
    let started = Instant::now();
    let mut total = 0.0;
    for _ in 0..500 {
        total += black_box(distribution(&values).unwrap()).median;
    }
    println!("profiled means ({total:.3}) in {:?}", started.elapsed());
}
