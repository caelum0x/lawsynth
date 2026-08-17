use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::infer_lagged_dependencies;
use std::{hint::black_box, time::Instant};
fn main() {
    let x = Identifier::new("x").unwrap();
    let y = Identifier::new("y").unwrap();
    let data = Dataset::new(
        TimeAxis::new((0..1_000).map(|i| i as f64).collect()).unwrap(),
        [
            NumericColumn::new(x, (0..1_000).map(|i| i as f64).collect()),
            NumericColumn::new(y, (0_u32..1_000).map(|i| i.saturating_sub(1) as f64).collect()),
        ],
    )
    .unwrap();
    let started = Instant::now();
    let mut edges = 0;
    for _ in 0..100 {
        edges += black_box(infer_lagged_dependencies(&data, 2, 0.9).unwrap()).edges.len();
    }
    println!("inferred {edges} edges in {:?}", started.elapsed());
}
