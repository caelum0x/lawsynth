use std::{hint::black_box, time::Instant};

use lawsynth_core::ResourceLimits;

fn main() {
    let limits = ResourceLimits::default();
    let started = Instant::now();
    for _ in 0..100_000 {
        black_box(limits.validate_dataset(10_000, 32)).expect("within defaults");
    }
    println!("resource validation: {:?}", started.elapsed());
}
