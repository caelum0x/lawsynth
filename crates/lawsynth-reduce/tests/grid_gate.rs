//! Honest gating: scattered (non-grid) data is reported as not reconstructed,
//! and error paths are surfaced rather than fabricating a reduction.

mod common;

use common::{axis, grid_dataset_2d};
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_reduce::{GridStatus, ReduceConfig, ReduceError, detect_reductions};

fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

#[test]
fn scattered_samples_are_not_reconstructed() {
    // x = t, y = t^2, f = arbitrary: not a Cartesian product of x and y.
    let time: Vec<f64> = (0..24).map(|i| i as f64 * 0.1 + 0.1).collect();
    let x = time.clone();
    let y: Vec<f64> = time.iter().map(|t| t * t).collect();
    let f: Vec<f64> = x.iter().zip(&y).map(|(a, b)| a + b).collect();
    let dataset = Dataset::new(
        TimeAxis::new(time).unwrap(),
        [
            NumericColumn::new(id("x"), x),
            NumericColumn::new(id("y"), y),
            NumericColumn::new(id("f"), f),
        ],
    )
    .unwrap();

    let report = detect_reductions(&dataset, &ReduceConfig::with_target("f")).unwrap();
    assert!(matches!(report.grid, GridStatus::NotReconstructed { .. }));
    assert!(report.is_empty());
}

#[test]
fn unknown_target_is_rejected() {
    let xs = axis(0.0, 0.25, 6);
    let ys = axis(0.0, 0.25, 6);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| x + y);
    let err = detect_reductions(&dataset, &ReduceConfig::with_target("missing")).unwrap_err();
    assert!(matches!(err, ReduceError::UnknownTarget { .. }));
}

#[test]
fn invalid_config_is_rejected() {
    let xs = axis(0.0, 0.25, 6);
    let ys = axis(0.0, 0.25, 6);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| x + y);
    let mut config = ReduceConfig::with_target("f");
    config.additive_tol = -1.0;
    let err = detect_reductions(&dataset, &config).unwrap_err();
    assert!(matches!(err, ReduceError::InvalidConfig { field: "additive_tol" }));
}
