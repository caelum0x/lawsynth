use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_dynamics::ControlledProblem;
use std::{hint::black_box, time::Instant};
fn main() {
    let x = Identifier::new("x").unwrap();
    let u = Identifier::new("u").unwrap();
    let data = Dataset::new(
        TimeAxis::new((0..1_000).map(|i| i as f64).collect()).unwrap(),
        [
            NumericColumn::new(x.clone(), (0..1_000).map(|i| i as f64).collect()),
            NumericColumn::new(u.clone(), vec![1.0; 1_000]),
        ],
    )
    .unwrap();
    let started = Instant::now();
    let mut n = 0;
    for _ in 0..10_000 {
        n += black_box(ControlledProblem::new(data.clone(), [x.clone()], [u.clone()]).unwrap())
            .inputs()
            .len();
    }
    println!("validated {n} controls in {:?}", started.elapsed());
}
