use lawsynth_causal::pearson_independence;
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let x: Vec<f64> = (0..100_000).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| 2.0 * v + 1.0).collect();
    let start = Instant::now();
    for _ in 0..100 {
        black_box(pearson_independence(&x, &y).unwrap());
    }
    println!("100 correlations in {:?}", start.elapsed());
}
