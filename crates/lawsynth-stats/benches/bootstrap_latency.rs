use lawsynth_stats::{BootstrapConfig, bootstrap_indices};
use std::{hint::black_box, time::Instant};

fn main() {
    let config = BootstrapConfig {
        replicates: 100,
        block_size: 8,
        seed: 42,
    };
    let started = Instant::now();
    let mut draws = 0;
    for _ in 0..1_000 {
        draws += black_box(bootstrap_indices(1_000, &config).unwrap())
            .iter()
            .map(Vec::len)
            .sum::<usize>();
    }
    println!(
        "generated {draws} bootstrap draws in {:?}",
        started.elapsed()
    );
}
