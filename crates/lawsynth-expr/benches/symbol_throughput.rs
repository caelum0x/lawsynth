use std::{hint::black_box, time::Instant};

use lawsynth_expr::{parse, symbols};

fn main() {
    let expression = parse("a * b + c * d + e * (a + d)").unwrap();
    let started = Instant::now();
    let mut total = 0;
    for _ in 0..100_000 {
        total += black_box(symbols(&expression)).len();
    }
    println!("collected {total} symbols in {:?}", started.elapsed());
}
