//! Symmetry demos: a translational (x − y) target is detected as a Difference
//! symmetry; a product target is detected as a Product symmetry.

mod common;

use common::{axis, grid_dataset_2d};
use lawsynth_reduce::{ReduceConfig, SymmetryKind, detect_reductions};

#[test]
fn detects_translational_difference_symmetry() {
    // f = (x - y)^2 depends only on (x - y).
    let xs = axis(0.0, 0.2, 12);
    let ys = axis(0.0, 0.2, 12);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| (x - y) * (x - y));

    let report = detect_reductions(&dataset, &ReduceConfig::with_target("f")).unwrap();
    assert!(report.grid.is_reconstructed());

    let diff = report
        .symmetries
        .iter()
        .find(|s| s.kind == SymmetryKind::Difference)
        .expect("difference symmetry should be detected");
    assert_eq!(diff.variables, ("x".to_string(), "y".to_string()));
    assert!(diff.residual < 1e-9, "difference residual too high: {}", diff.residual);

    // The other symmetries must NOT fire for (x - y)^2.
    assert!(
        !report.symmetries.iter().any(|s| matches!(
            s.kind,
            SymmetryKind::Sum | SymmetryKind::Product | SymmetryKind::Ratio
        )),
        "only the difference symmetry should be reported"
    );
    // (x - y)^2 is not additively separable (mixed partial = -2).
    assert!(report.separabilities.is_empty());
}

#[test]
fn detects_product_symmetry() {
    // f = (x * y)^2 depends only on (x * y): x*f_x - y*f_y = 0 exactly.
    let xs = axis(0.5, 0.2, 12);
    let ys = axis(0.5, 0.2, 12);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| (x * y) * (x * y));

    let report = detect_reductions(&dataset, &ReduceConfig::with_target("f")).unwrap();
    let product = report
        .symmetries
        .iter()
        .find(|s| s.kind == SymmetryKind::Product)
        .expect("product symmetry should be detected");
    assert!(product.residual < 1e-9, "product residual too high: {}", product.residual);
}
