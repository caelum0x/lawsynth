use lawsynth_bundle::BundleSignature;
use std::{hint::black_box, time::Instant};

fn main() {
    let payload = vec![0x5a; 1 << 16];
    let started = Instant::now();
    let mut bytes = 0;
    for _ in 0..1_000 {
        bytes += black_box(BundleSignature::authenticate(b"benchmark-key", &payload))
            .0
            .len();
    }
    println!("computed {bytes} tag characters in {:?}", started.elapsed());
}
