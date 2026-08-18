use lawsynth_causal::{CausalConfig, granger_test};
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let x: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.03).sin()).collect();
    let mut y = vec![0.0; 1000];
    for i in 1..1000 {
        y[i] = 0.6 * y[i - 1] + x[i - 1];
    }
    let start = Instant::now();
    for _ in 0..100 {
        black_box(
            granger_test(
                &x,
                &y,
                CausalConfig { max_lag: 2, min_samples: 20, ..Default::default() },
            )
            .unwrap(),
        );
    }
    println!("100 Granger fits in {:?}", start.elapsed());
}
