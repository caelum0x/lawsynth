use lawsynth_uncertainty::{BootstrapConfig, Samples, bootstrap};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let samples = Samples::new((0..1_000).map(|i| (i as f64 / 20.0).sin()).collect()).unwrap();
    let started = Instant::now();
    let result = bootstrap(
        &samples,
        BootstrapConfig {
            replicates: 1_000,
            seed: 9,
        },
        |draw| draw.iter().sum::<f64>() / draw.len() as f64,
    )
    .unwrap();
    black_box(result);
    println!("bootstrap 1,000x1,000: {:?}", started.elapsed());
}
