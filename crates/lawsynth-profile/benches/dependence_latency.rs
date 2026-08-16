use lawsynth_profile::pearson_correlation;
use std::{hint::black_box, time::Instant};

fn main() {
    let left = (0..10_000).map(|i| i as f64 * 0.001).collect::<Vec<_>>();
    let right = left.iter().map(|v| 3.0 * v + 1.0).collect::<Vec<_>>();
    let started = Instant::now();
    let mut sum = 0.0;
    for _ in 0..1_000 {
        sum += black_box(pearson_correlation(&left, &right).unwrap());
    }
    println!(
        "computed correlations ({sum:.1}) in {:?}",
        started.elapsed()
    );
}
