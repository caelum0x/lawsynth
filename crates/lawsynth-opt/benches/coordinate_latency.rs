use lawsynth_opt::{CoordinateConfig, ParameterBounds, coordinate_minimize};
use std::{hint::black_box, time::Instant};
fn main() {
    let started = Instant::now();
    let mut total = 0.0;
    for _ in 0..10_000 {
        total += black_box(
            coordinate_minimize(
                &[4.0, -4.0],
                ParameterBounds::new(-5.0, 5.0).unwrap(),
                CoordinateConfig::default(),
                |p| (p[0] - 1.0).powi(2) + (p[1] + 2.0).powi(2),
            )
            .unwrap(),
        )
        .objective;
    }
    println!("coordinate objective {total} in {:?}", started.elapsed());
}
