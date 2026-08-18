use lawsynth_bundle::canonical_entry_order;
use std::{hint::black_box, time::Instant};

fn main() {
    let paths = (0..1_000).map(|i| format!("laws/{i:04}/equation.bin")).collect::<Vec<_>>();
    let started = Instant::now();
    let mut entries = 0;
    for _ in 0..1_000 {
        entries += black_box(canonical_entry_order(paths.clone()).unwrap()).len();
    }
    println!("canonicalized {entries} paths in {:?}", started.elapsed());
}
