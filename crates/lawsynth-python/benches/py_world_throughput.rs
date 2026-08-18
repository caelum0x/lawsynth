#[path = "../src/convert.rs"]
mod convert;

use convert::identifier_values;
use std::{hint::black_box, time::Instant};
fn main() {
    let pairs = (0..100).map(|i| (format!("x{i}"), i as f64)).collect::<Vec<_>>();
    let started = Instant::now();
    let mut values = 0;
    for _ in 0..100_000 {
        values += black_box(identifier_values(pairs.clone().into_iter().collect()).unwrap()).len();
    }
    println!("converted {values} Python-map values in {:?}", started.elapsed());
}
