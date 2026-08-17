use lawsynth_cli::parse_identifier_list;
use std::{hint::black_box, time::Instant};
fn main() {
    let source = (0..1_000).map(|i| format!("x{i}")).collect::<Vec<_>>().join(",");
    let started = Instant::now();
    let mut ids = 0;
    for _ in 0..10_000 {
        ids += black_box(parse_identifier_list(&source).unwrap()).len();
    }
    println!("parsed {ids} identifiers in {:?}", started.elapsed());
}
