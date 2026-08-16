use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_preprocess::standardize;
use std::{hint::black_box, time::Instant};

fn main() {
    let values = (0..10_000).map(|i| i as f64).collect();
    let data = Dataset::new(
        TimeAxis::new((0..10_000).map(|i| i as f64).collect()).unwrap(),
        [NumericColumn::new(Identifier::new("x").unwrap(), values)],
    )
    .unwrap();
    let started = Instant::now();
    let mut rows = 0;
    for _ in 0..100 {
        rows += black_box(standardize(&data).unwrap()).0.time().len();
    }
    println!("standardized {rows} rows in {:?}", started.elapsed());
}
