//! Shared helpers for structural-reduction integration tests.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// Builds a `Dataset` from `f(x, y)` sampled over a Cartesian grid, flattened in
/// row-major order with a synthetic monotonic time index.
pub fn grid_dataset_2d(xs: &[f64], ys: &[f64], f: impl Fn(f64, f64) -> f64) -> Dataset {
    let (mut xc, mut yc, mut fc) = (Vec::new(), Vec::new(), Vec::new());
    for &x in xs {
        for &y in ys {
            xc.push(x);
            yc.push(y);
            fc.push(f(x, y));
        }
    }
    let time: Vec<f64> = (0..xc.len()).map(|i| i as f64).collect();
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [
            NumericColumn::new(id("x"), xc),
            NumericColumn::new(id("y"), yc),
            NumericColumn::new(id("f"), fc),
        ],
    )
    .unwrap()
}

/// A standard evenly spaced axis of `n` points from `start` with `step`.
pub fn axis(start: f64, step: f64, n: usize) -> Vec<f64> {
    (0..n).map(|i| start + step * i as f64).collect()
}
