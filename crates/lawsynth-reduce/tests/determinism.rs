//! Determinism: identical inputs yield a bit-identical report on replay.

mod common;

use common::{axis, grid_dataset_2d};
use lawsynth_reduce::{ReduceConfig, detect_reductions};

#[test]
fn replay_is_bit_identical() {
    let xs = axis(0.0, 0.25, 12);
    let ys = axis(0.0, 0.25, 12);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| x.sin() + y * y);
    let config = ReduceConfig::with_target("f");

    let first = detect_reductions(&dataset, &config).unwrap();
    let second = detect_reductions(&dataset, &config).unwrap();
    assert_eq!(first, second);

    // Residual bit patterns must match exactly, not merely be close.
    let a = &first.separabilities[0];
    let b = &second.separabilities[0];
    assert_eq!(a.reconstruction_residual.to_bits(), b.reconstruction_residual.to_bits());
    assert_eq!(a.screening_residual.to_bits(), b.screening_residual.to_bits());
    assert_eq!(a.confidence.to_bits(), b.confidence.to_bits());
}

#[test]
fn determinism_holds_for_symmetric_target() {
    let xs = axis(0.0, 0.2, 12);
    let ys = axis(0.0, 0.2, 12);
    let dataset = grid_dataset_2d(&xs, &ys, |x, y| (x - y) * (x - y));
    let config = ReduceConfig::with_target("f");
    assert_eq!(
        detect_reductions(&dataset, &config).unwrap(),
        detect_reductions(&dataset, &config).unwrap()
    );
}
