use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_features::FeatureLibrary;
use std::{hint::black_box, time::Instant};

fn main() {
    let x = Identifier::new("x").unwrap();
    let data = Dataset::new(
        TimeAxis::new((0..2_000).map(|i| i as f64).collect()).unwrap(),
        [NumericColumn::new(
            x.clone(),
            (0..2_000).map(|i| 1.0 + i as f64 / 100.0).collect(),
        )],
    )
    .unwrap();
    let library = FeatureLibrary::bounded_rational([x]).unwrap();
    let started = Instant::now();
    let mut rows = 0;
    for _ in 0..100 {
        rows += black_box(library.evaluate(&data).unwrap()).rows.len();
    }
    println!(
        "evaluated {rows} rational feature rows in {:?}",
        started.elapsed()
    );
}
