use lawsynth_sparse::{LassoConfig, RegressionProblem, lasso};
use std::{hint::black_box, time::Instant};

fn main() {
    let problem = RegressionProblem::new(
        (0..1_000)
            .map(|i| {
                let x = i as f64 / 1_000.0;
                vec![1.0, x, x * x]
            })
            .collect(),
        (0..1_000).map(|i| 1.0 + 2.0 * i as f64 / 1_000.0).collect(),
    )
    .unwrap();
    let config = LassoConfig::default();
    let started = Instant::now();
    let mut terms = 0;
    for _ in 0..100 {
        terms += black_box(lasso(&problem, &config).unwrap()).coefficients.len();
    }
    println!("fit {terms} lasso coefficients in {:?}", started.elapsed());
}
