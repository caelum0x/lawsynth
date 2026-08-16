use lawsynth_differentiate::spectral_derivative;
use std::{hint::black_box, time::Instant};

fn main() {
    let time = (0..2_048)
        .map(|i| i as f64 * std::f64::consts::TAU / 2_048.0)
        .collect::<Vec<_>>();
    let values = (0..2_048)
        .map(|i| ((i as f64) * std::f64::consts::TAU / 2_048.0).sin())
        .collect::<Vec<_>>();
    let started = Instant::now();
    let mut len = 0;
    for _ in 0..100 {
        len += black_box(spectral_derivative(&time, &values).unwrap()).len();
    }
    println!(
        "differentiated {len} spectral samples in {:?}",
        started.elapsed()
    );
}
