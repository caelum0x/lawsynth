use lawsynth_cli::parse_assignment_text;
use std::{hint::black_box, time::Instant};
fn main() {
    let started = Instant::now();
    let mut values = 0.0;
    for _ in 0..1_000_000 {
        values += black_box(parse_assignment_text("growth=1.25").unwrap()).1;
    }
    println!(
        "parsed assignment total {values} in {:?}",
        started.elapsed()
    );
}
