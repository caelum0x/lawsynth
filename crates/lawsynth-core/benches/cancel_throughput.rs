use std::{hint::black_box, time::Instant};

use lawsynth_core::CancellationToken;

fn main() {
    let token = CancellationToken::default();
    let started = Instant::now();
    let mut observed = 0_u64;
    for _ in 0..1_000_000 {
        observed += u64::from(black_box(token.is_cancelled()));
    }
    token.cancel();
    assert!(token.is_cancelled());
    println!(
        "cancel checks: {:?} ({observed} observed cancellations before signalling)",
        started.elapsed()
    );
}
