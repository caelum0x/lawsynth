use lawsynth_differentiate::tvreg_series;
use std::{hint::black_box, time::Instant};

fn main() {
    let time = (0..2_000).map(|i| i as f64 * 0.01).collect::<Vec<_>>();
    let values = time.iter().map(|t| t.sin()).collect::<Vec<_>>();
    let started = Instant::now();
    let mut len = 0;
    for _ in 0..100 {
        len += black_box(tvreg_series(&time, &values, 0.05, 40).unwrap()).len();
    }
    println!("regularized {len} derivative samples in {:?}", started.elapsed());
}
