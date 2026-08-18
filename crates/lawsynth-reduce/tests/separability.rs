//! Separability demos: a known additive and a known multiplicative target are
//! detected with the correct partition and a low residual.

mod common;

use common::{axis, grid_dataset_2d};
use lawsynth_reduce::{ReduceConfig, SeparabilityKind, detect_reductions};

#[test]
fn detects_additive_separability_sin_x_plus_y_squared() {
    // f = sin(x) + y^2  ->  additive, partition {x} | {y}.
    let xs = axis(0.0, 0.25, 12);
    let ys = axis(0.0, 0.25, 12);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| x.sin() + y * y);

    let report = detect_reductions(&dataset, &ReduceConfig::with_target("f")).unwrap();
    assert!(report.grid.is_reconstructed());

    let additive = report
        .separabilities
        .iter()
        .find(|s| s.kind == SeparabilityKind::Additive)
        .expect("additive separability should be detected");
    assert_eq!(additive.group_a, vec!["x".to_string()]);
    assert_eq!(additive.group_b, vec!["y".to_string()]);
    assert!(
        additive.reconstruction_residual < 1e-6,
        "reconstruction residual too high: {}",
        additive.reconstruction_residual
    );
    assert!(additive.confidence > 0.999);

    // It is NOT multiplicative.
    assert!(
        !report.separabilities.iter().any(|s| s.kind == SeparabilityKind::Multiplicative),
        "additive target must not be flagged multiplicative"
    );
}

#[test]
fn detects_multiplicative_separability_x_times_exp_y() {
    // f = x * e^y (x > 0)  ->  multiplicative, partition {x} | {y}.
    let xs = axis(1.0, 0.2, 12);
    let ys = axis(0.0, 0.2, 12);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| x * y.exp());

    let report = detect_reductions(&dataset, &ReduceConfig::with_target("f")).unwrap();

    let mult = report
        .separabilities
        .iter()
        .find(|s| s.kind == SeparabilityKind::Multiplicative)
        .expect("multiplicative separability should be detected");
    assert_eq!(mult.group_a, vec!["x".to_string()]);
    assert_eq!(mult.group_b, vec!["y".to_string()]);
    assert!(
        mult.reconstruction_residual < 1e-6,
        "reconstruction residual too high: {}",
        mult.reconstruction_residual
    );

    // A multiplicative-but-not-additive target must not be flagged additive.
    assert!(
        !report.separabilities.iter().any(|s| s.kind == SeparabilityKind::Additive),
        "x*e^y is not additively separable"
    );
}

#[test]
fn default_target_uses_last_column_when_unset() {
    // With no explicit target, the lexicographically greatest column is used.
    // Columns sort as f < x < y, so the default target is `y` (an input axis),
    // which yields a valid grid but no meaningful reduction — still deterministic.
    let xs = axis(0.0, 0.25, 8);
    let ys = axis(0.0, 0.25, 8);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| x.sin() + y * y);
    let report = detect_reductions(&dataset, &ReduceConfig::default()).unwrap();
    assert_eq!(report.target, "y");
}
