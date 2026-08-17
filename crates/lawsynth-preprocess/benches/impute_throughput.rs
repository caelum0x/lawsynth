use lawsynth_preprocess::{ImputationMethod, impute_series};
use std::{hint::black_box, time::Instant};

fn main() {
    let time = (0..10_000).map(|i| i as f64).collect::<Vec<_>>();
    let values =
        time.iter().enumerate().map(|(i, t)| (i % 23 != 0).then_some(*t * 0.5)).collect::<Vec<_>>();
    let started = Instant::now();
    let mut filled = 0;
    for _ in 0..100 {
        filled += black_box(impute_series(&time, &values, ImputationMethod::Mean).unwrap()).0.len();
    }
    println!("imputed {filled} values in {:?}", started.elapsed());
}
