//! Negative demo: a genuinely non-separable, non-symmetric target must not
//! trigger any spurious reduction above tolerance.

mod common;

use common::{axis, grid_dataset_2d};
use lawsynth_reduce::{ReduceConfig, detect_reductions};

#[test]
fn does_not_falsely_flag_x_y_plus_sin_x_plus_y() {
    // f = x*y + sin(x + y): mixed partial = 1 - sin(x+y) != 0, and none of the
    // difference/sum/product/ratio symmetries hold.
    let xs = axis(0.5, 0.2, 12);
    let ys = axis(0.5, 0.2, 12);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| x * y + (x + y).sin());

    let report = detect_reductions(&dataset, &ReduceConfig::with_target("f")).unwrap();
    assert!(report.grid.is_reconstructed());
    assert!(
        report.separabilities.is_empty(),
        "no separability should be reported, got {:?}",
        report.separabilities
    );
    assert!(
        report.symmetries.is_empty(),
        "no symmetry should be reported, got {:?}",
        report.symmetries
    );
    assert!(report.is_empty());
}

#[test]
fn does_not_flag_additive_target_as_symmetric() {
    // f = sin(x) + y^2 is separable but has no x±y / x*y / x/y symmetry.
    let xs = axis(0.0, 0.25, 12);
    let ys = axis(0.0, 0.25, 12);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| x.sin() + y * y);
    let report = detect_reductions(&dataset, &ReduceConfig::with_target("f")).unwrap();
    assert!(report.symmetries.is_empty(), "unexpected symmetry: {:?}", report.symmetries);
}
