use std::{hint::black_box, time::Instant};

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

fn dataset() -> Dataset {
    let time = (0..10_000).map(|i| i as f64 * 0.01).collect();
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [NumericColumn::new(
            Identifier::new("x").unwrap(),
            (0..10_000).map(|i| (i as f64).sin()).collect(),
        )],
    )
    .unwrap()
}

fn main() {
    let data = dataset();
    let started = Instant::now();
    let mut rows = 0;
    for _ in 0..200 {
        rows += black_box(data.batches(256).unwrap())
            .iter()
            .map(|batch| batch.time.len())
            .sum::<usize>();
    }
    println!(
        "materialized {rows} aligned batch rows in {:?}",
        started.elapsed()
    );
}
