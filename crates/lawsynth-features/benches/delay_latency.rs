use lawsynth_features::delayed_columns;
use std::{hint::black_box, time::Instant};

fn main() {
    let values = (0..10_000).map(|i| i as f64).collect::<Vec<_>>();
    let started = Instant::now();
    let mut rows = 0;
    for _ in 0..100 {
        rows += black_box(delayed_columns(&values, &[0, 1, 8]).unwrap())
            .rows
            .len();
    }
    println!("embedded {rows} delayed rows in {:?}", started.elapsed());
}
