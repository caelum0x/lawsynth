use lawsynth_sparse::{GroupConfig, RegressionProblem, group_stlsq};
use std::{hint::black_box, time::Instant};

fn main() {
    let problem = RegressionProblem::new(
        (0..1_000)
            .map(|i| {
                let x = i as f64 / 1_000.0;
                vec![1.0, x, x * x, x * x * x]
            })
            .collect(),
        (0..1_000).map(|i| 1.0 + 2.0 * i as f64 / 1_000.0).collect(),
    )
    .unwrap();
    let config = GroupConfig::default();
    let started = Instant::now();
    let mut terms = 0;
    for _ in 0..100 {
        terms += black_box(group_stlsq(&problem, &[vec![0, 1], vec![2, 3]], &config).unwrap())
            .coefficients
            .len();
    }
    println!("fit {terms} grouped coefficients in {:?}", started.elapsed());
}
