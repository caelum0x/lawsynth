//! Honest closed-loop tests: discover a controlled model, roll it forward under
//! a deterministic control, and score the rollout against ground truth.
//!
//! These exercise the full loop `discover → simulate → score`, plus
//! determinism, score discrimination, and the input-validation error paths.

mod common;

use common::{control_signal, id, oscillator_dataset};
use lawsynth_control::{
    ControlConfig, ControlError, ControlSignal, ControlSpec, SimConfig, ValidationConfig,
    discover_controlled, simulate_controlled, validate_controlled,
};
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_score::fit_statistics;

/// Discovers the standard forced oscillator and returns `(model, dataset, spec)`.
fn discovered() -> (lawsynth_control::ControlledModel, Dataset, ControlSpec) {
    let dataset = oscillator_dataset();
    let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
    let model = discover_controlled(&dataset, &spec, &ControlConfig::default()).unwrap();
    (model, dataset, spec)
}

/// Bit-comparison helper for whole trajectories / score fields.
fn bits(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.to_bits()).collect()
}

/// The discovered model, rolled forward under the dataset's OWN control on the
/// dataset's grid, reproduces the observed states with R² ≥ 0.999.
#[test]
fn closed_loop_validation_scores_near_one() {
    let (model, dataset, spec) = discovered();
    let score = validate_controlled(&model, &dataset, &spec, &ValidationConfig::default()).unwrap();

    assert!(
        score.aggregate_r_squared >= 0.999,
        "aggregate R² was {} (< 0.999)",
        score.aggregate_r_squared
    );
    for state in &model.states {
        let per = score.state_score(state).unwrap();
        assert!(per.r_squared >= 0.999, "state {state} R² was {} (< 0.999)", per.r_squared);
    }
}

/// The same rollout, driven by the exact multi-sine *closure* (no interpolation)
/// from the true initial condition, matches the ground-truth trajectory tightly.
#[test]
fn closed_loop_simulation_matches_ground_truth() {
    let (model, dataset, _spec) = discovered();

    let control = ControlSignal::from_fn([id("u")], |t| vec![control_signal(t)]).unwrap();
    let config = SimConfig::new(0.0, 0.005, 4000).unwrap();
    let trajectory = simulate_controlled(&model, &[1.0, 0.0], &control, &config).unwrap();

    for state in ["x", "y"] {
        let observed = dataset.columns()[&id(state)].values.as_slice();
        let predicted = trajectory.column(&id(state)).unwrap();
        let stats = fit_statistics(observed, predicted).unwrap();
        assert!(
            stats.r_squared >= 0.999,
            "state {state} closure-driven R² was {}",
            stats.r_squared
        );
    }
}

/// A deliberately wrong model — the control coefficient zeroed out — scores
/// materially worse, proving the score discriminates model quality.
#[test]
fn wrong_model_scores_materially_worse() {
    let (model, dataset, spec) = discovered();
    let good = validate_controlled(&model, &dataset, &spec, &ValidationConfig::default()).unwrap();

    // Zero the `u` coefficient in the ẏ equation: the control's real forcing is
    // now unmodelled, so the open-loop rollout must drift away from the truth.
    let mut wrong = model.clone();
    let u_index = wrong.library_terms.iter().position(|label| label == "u").unwrap();
    let y_equation = wrong.equations.iter_mut().find(|equation| equation.state == id("y")).unwrap();
    y_equation.coefficients[u_index] = 0.0;

    let bad = validate_controlled(&wrong, &dataset, &spec, &ValidationConfig::default()).unwrap();

    assert!(
        bad.aggregate_r_squared < good.aggregate_r_squared - 0.05,
        "zeroing the control barely changed R²: good {} vs bad {}",
        good.aggregate_r_squared,
        bad.aggregate_r_squared
    );
    assert!(
        bad.aggregate_rmse > good.aggregate_rmse * 5.0,
        "zeroing the control barely changed RMSE: good {} vs bad {}",
        good.aggregate_rmse,
        bad.aggregate_rmse
    );
}

/// Identical inputs yield a bit-identical trajectory.
#[test]
fn simulation_is_bit_identical() {
    let (model, _dataset, _spec) = discovered();
    let make = || ControlSignal::from_fn([id("u")], |t| vec![control_signal(t)]).unwrap();
    let config = SimConfig::new(0.0, 0.01, 500).unwrap();

    let first = simulate_controlled(&model, &[1.0, 0.0], &make(), &config).unwrap();
    let second = simulate_controlled(&model, &[1.0, 0.0], &make(), &config).unwrap();

    assert_eq!(first.time, second.time);
    for state in &model.states {
        assert_eq!(
            bits(first.column(state).unwrap()),
            bits(second.column(state).unwrap()),
            "trajectory for {state} was not bit-identical"
        );
    }
}

/// Identical inputs yield a bit-identical validation score.
#[test]
fn validation_is_bit_identical() {
    let (model, dataset, spec) = discovered();
    let first = validate_controlled(&model, &dataset, &spec, &ValidationConfig::default()).unwrap();
    let second =
        validate_controlled(&model, &dataset, &spec, &ValidationConfig::default()).unwrap();

    assert_eq!(first.aggregate_r_squared.to_bits(), second.aggregate_r_squared.to_bits());
    assert_eq!(first.aggregate_rmse.to_bits(), second.aggregate_rmse.to_bits());
    for (a, b) in first.per_state.iter().zip(&second.per_state) {
        assert_eq!(a.r_squared.to_bits(), b.r_squared.to_bits());
        assert_eq!(a.rmse.to_bits(), b.rmse.to_bits());
    }
}

/// A sampled control on a fine grid gives essentially the same rollout as the
/// exact closure — evidence the linear interpolation rule is sound.
#[test]
fn sampled_and_closure_controls_agree() {
    let (model, _dataset, _spec) = discovered();

    let config = SimConfig::new(0.0, 0.005, 2000).unwrap();
    let closure = ControlSignal::from_fn([id("u")], |t| vec![control_signal(t)]).unwrap();
    let closed = simulate_controlled(&model, &[1.0, 0.0], &closure, &config).unwrap();

    // Sample the same control densely and interpolate.
    let sample_times = (0..=2000).map(|i| i as f64 * 0.005).collect::<Vec<_>>();
    let sample_u = sample_times.iter().map(|t| control_signal(*t)).collect::<Vec<_>>();
    let sampled = ControlSignal::sampled([id("u")], sample_times, vec![sample_u]).unwrap();
    let sampled_traj = simulate_controlled(&model, &[1.0, 0.0], &sampled, &config).unwrap();

    for state in ["x", "y"] {
        let a = closed.column(&id(state)).unwrap();
        let b = sampled_traj.column(&id(state)).unwrap();
        let max_diff = a.iter().zip(b).map(|(p, q)| (p - q).abs()).fold(0.0_f64, f64::max);
        // Linear interpolation at the RK4 midpoints introduces a small,
        // deterministic error versus the exact closure; on this fine grid it
        // stays well under 1e-4, confirming the interpolation rule is sound.
        assert!(max_diff < 1e-4, "sampled vs closure diverged on {state} by {max_diff}");
    }
}

/// Finer sub-stepping keeps the score high and does not degrade the fit.
#[test]
fn substep_refinement_keeps_score_high() {
    let (model, dataset, spec) = discovered();
    let coarse =
        validate_controlled(&model, &dataset, &spec, &ValidationConfig { substeps: 1 }).unwrap();
    let fine =
        validate_controlled(&model, &dataset, &spec, &ValidationConfig { substeps: 4 }).unwrap();
    assert!(coarse.aggregate_r_squared >= 0.999);
    assert!(fine.aggregate_r_squared >= 0.999);
}

/// Error path: an initial-state vector of the wrong length is rejected.
#[test]
fn rejects_initial_state_dimension_mismatch() {
    let (model, _dataset, _spec) = discovered();
    let control = ControlSignal::from_fn([id("u")], |t| vec![control_signal(t)]).unwrap();
    let config = SimConfig::new(0.0, 0.01, 10).unwrap();
    let error = simulate_controlled(&model, &[1.0], &control, &config).unwrap_err();
    assert_eq!(error, ControlError::InitialStateDimension { expected: 2, found: 1 });
}

/// Error path: a control signal that names the wrong channel is rejected.
#[test]
fn rejects_control_channel_mismatch() {
    let (model, _dataset, _spec) = discovered();
    let control = ControlSignal::from_fn([id("w")], |t| vec![control_signal(t)]).unwrap();
    let config = SimConfig::new(0.0, 0.01, 10).unwrap();
    let error = simulate_controlled(&model, &[1.0, 0.0], &control, &config).unwrap_err();
    assert!(
        matches!(error, ControlError::ControlMismatch { .. }),
        "expected a control mismatch, got {error:?}"
    );
}

/// Error path: a sampled control whose column length disagrees with its time
/// axis is rejected at construction (control grid mismatch).
#[test]
fn rejects_sampled_control_grid_mismatch() {
    let error =
        ControlSignal::sampled([id("u")], vec![0.0, 0.1, 0.2], vec![vec![1.0, 2.0]]).unwrap_err();
    assert!(
        matches!(error, ControlError::ControlGrid(_)),
        "expected a control grid error, got {error:?}"
    );
}

/// Error path: a non-positive RK4 step is rejected.
#[test]
fn rejects_nonpositive_step() {
    let error = SimConfig::new(0.0, 0.0, 10).unwrap_err();
    assert!(matches!(error, ControlError::ControlGrid(_)), "got {error:?}");
}

/// Error path: validation needs at least two samples to form a step.
#[test]
fn rejects_single_sample_validation() {
    let dataset = Dataset::new(
        TimeAxis::new(vec![0.0]).unwrap(),
        [NumericColumn::new(id("x"), vec![1.0]), NumericColumn::new(id("u"), vec![0.5])],
    )
    .unwrap();
    let spec = ControlSpec::new([id("x")], [id("u")]).unwrap();
    // A one-sample dataset cannot be differentiated, so discovery itself fails;
    // build a model on real data but validate it against the tiny dataset.
    let (model, _d, _s) = discovered();
    let _ = &model;
    // Use a spec/model consistent with the tiny dataset via a fresh discovery
    // on a minimal but differentiable dataset, then swap in the 1-row dataset.
    let train = Dataset::new(
        TimeAxis::new(vec![0.0, 0.1, 0.2, 0.3]).unwrap(),
        [
            NumericColumn::new(id("x"), vec![0.0, 0.1, 0.2, 0.3]),
            NumericColumn::new(id("u"), vec![0.1, 0.2, 0.1, 0.2]),
        ],
    )
    .unwrap();
    let tiny_model = discover_controlled(&train, &spec, &ControlConfig::default()).unwrap();
    let error = validate_controlled(&tiny_model, &dataset, &spec, &ValidationConfig::default())
        .unwrap_err();
    assert!(matches!(error, ControlError::ControlGrid(_)), "got {error:?}");
}

/// Error path: a spec disagreeing with the model is rejected by validation.
#[test]
fn rejects_spec_model_mismatch() {
    let (model, dataset, _spec) = discovered();
    // Swap state order so the spec no longer matches the model.
    let mismatched = ControlSpec::new([id("y"), id("x")], [id("u")]).unwrap();
    let error = validate_controlled(&model, &dataset, &mismatched, &ValidationConfig::default())
        .unwrap_err();
    assert!(matches!(error, ControlError::ControlMismatch { .. }), "got {error:?}");
}
