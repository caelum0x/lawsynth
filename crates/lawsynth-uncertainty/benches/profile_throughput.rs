use lawsynth_uncertainty::{IntervalConfig, ProfilePoint, profile_quadratic};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let points: Vec<ProfilePoint> = (-100..=100)
        .map(|i| {
            let x = i as f64 / 20.0;
            ProfilePoint {
                parameter: x,
                objective: (x - 1.25).powi(2) * 2.0 + 7.0,
            }
        })
        .collect();
    let started = Instant::now();
    for _ in 0..10_000 {
        black_box(profile_quadratic(&points, IntervalConfig::default()).unwrap());
    }
    println!("10,000 quadratic profiles: {:?}", started.elapsed());
}
