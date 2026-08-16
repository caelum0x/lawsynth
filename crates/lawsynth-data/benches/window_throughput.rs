use std::{hint::black_box, time::Instant};

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis, WindowConfig};

fn main() {
    let data = Dataset::new(
        TimeAxis::new((0..4_000).map(|i| i as f64).collect()).unwrap(),
        [NumericColumn::new(
            Identifier::new("signal").unwrap(),
            (0..4_000).map(|i| (i as f64).cos()).collect(),
        )],
    )
    .unwrap();
    let started = Instant::now();
    let mut count = 0;
    for _ in 0..100 {
        count += black_box(data.windows(WindowConfig::new(32, 8)).unwrap()).len();
    }
    println!("created {count} windows in {:?}", started.elapsed());
}
