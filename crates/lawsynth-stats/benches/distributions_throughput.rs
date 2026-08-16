use lawsynth_stats::{normal_cdf, normal_pdf};
use std::{hint::black_box, time::Instant};

fn main() {
    let started = Instant::now();
    let mut sum = 0.0;
    for i in 0..1_000_000 {
        let x = i as f64 / 100_000.0 - 5.0;
        sum += black_box(normal_pdf(x, 0.0, 1.0).unwrap() + normal_cdf(x, 0.0, 1.0).unwrap());
    }
    println!(
        "evaluated normal primitives ({sum:.3}) in {:?}",
        started.elapsed()
    );
}
