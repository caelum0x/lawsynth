use std::{hint::black_box, time::Instant};

use lawsynth_units::{convert, parse_unit};

fn main() {
    let from = parse_unit("km").unwrap();
    let to = parse_unit("m").unwrap();
    let started = Instant::now();
    let mut result = 0.0;
    for _ in 0..1_000_000 {
        result += black_box(convert(1.25, &from, &to).unwrap());
    }
    println!("converted total {result} in {:?}", started.elapsed());
}
