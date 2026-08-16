use lawsynth_opt::{NelderMeadConfig, ParameterBounds, nelder_mead_minimize};
use std::{hint::black_box, time::Instant};
fn main() {
    let started = Instant::now();
    let mut total = 0.0;
    for _ in 0..1_000 {
        let point = black_box(
            nelder_mead_minimize(
                &[3.0, -3.0],
                ParameterBounds::new(-5.0, 5.0).unwrap(),
                NelderMeadConfig::default(),
                |p| (p[0] - 1.0).powi(2) + (p[1] + 2.0).powi(2),
            )
            .unwrap(),
        );
        total += (point[0] - 1.0).powi(2) + (point[1] + 2.0).powi(2);
    }
    println!("simplex objective {total} in {:?}", started.elapsed());
}
