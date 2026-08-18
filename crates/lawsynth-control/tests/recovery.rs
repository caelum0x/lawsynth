//! Honest recovery tests for controlled (SINDYc) discovery on a known forced
//! system integrated with fine RK4 and a deterministic multi-sine control.

mod common;

use common::{
    OSCILLATOR_C, OSCILLATOR_CONTROL_GAIN, OSCILLATOR_K, coefficient_for, id, oscillator_dataset,
};
use lawsynth_control::{ControlConfig, ControlSpec, discover_controlled};
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_differentiate::differentiate_series;
use lawsynth_features::FeatureLibrary;
use lawsynth_sparse::{RegressionProblem, stlsq_standardized};

/// The forced oscillator `ẋ = y`, `ẏ = -k·x - c·y + u` is recovered — including
/// the control term — to a tight tolerance from RK4 data.
#[test]
fn recovers_forced_linear_oscillator() {
    let dataset = oscillator_dataset();
    let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
    let model = discover_controlled(&dataset, &spec, &ControlConfig::default()).unwrap();

    // ẋ = y : only the `y` term is active, with coefficient 1.
    let dx_y = coefficient_for(&model, "x", "y");
    assert!((dx_y - 1.0).abs() < 1e-2, "ẋ coefficient of y was {dx_y}");

    // ẏ = -k·x - c·y + gain·u : all three terms recovered.
    let dy_x = coefficient_for(&model, "y", "x");
    let dy_y = coefficient_for(&model, "y", "y");
    let dy_u = coefficient_for(&model, "y", "u");
    assert!((dy_x + OSCILLATOR_K).abs() < 1e-2, "ẏ coefficient of x was {dy_x}");
    assert!((dy_y + OSCILLATOR_C).abs() < 1e-2, "ẏ coefficient of y was {dy_y}");
    assert!((dy_u - OSCILLATOR_CONTROL_GAIN).abs() < 1e-2, "ẏ coefficient of u was {dy_u}");

    // Spurious cross/quadratic terms stay negligible in the ẏ equation.
    for (label, coefficient) in model.equation(&id("y")).unwrap().active_terms(&model.library_terms)
    {
        if !matches!(label, "x" | "y" | "u") {
            assert!(coefficient.abs() < 1e-2, "unexpected active term '{label}' = {coefficient}");
        }
    }
}

/// Dropping the control from the library leaves the control's real contribution
/// unmodelled, so the `ẏ` fit is materially worse. This proves the control term
/// is genuinely necessary, not an artefact.
#[test]
fn control_term_is_necessary() {
    let dataset = oscillator_dataset();
    let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
    let controlled = discover_controlled(&dataset, &spec, &ControlConfig::default()).unwrap();
    let controlled_rss = controlled.equation(&id("y")).unwrap().residual_sum_squares;

    // Fit the SAME ẏ target against a states-only polynomial library (no u).
    let time = dataset.time().values();
    let y_values = &dataset.columns()[&id("y")].values;
    let dy = differentiate_series(time, y_values).unwrap();

    let states_only = FeatureLibrary::polynomial([id("x"), id("y")], 2, true).unwrap();
    let matrix = states_only.evaluate(&dataset).unwrap();
    let problem = RegressionProblem::new(matrix.rows, dy).unwrap();
    let states_only_solution =
        stlsq_standardized(&problem, &ControlConfig::default().sparse).unwrap();
    let states_only_rss = states_only_solution.residual_sum_squares;

    // The control carries real variance (RMS ~0.4), so omitting it inflates the
    // residual by orders of magnitude.
    assert!(
        states_only_rss > 100.0 * controlled_rss,
        "states-only RSS {states_only_rss} was not materially worse than controlled RSS {controlled_rss}"
    );
}

/// A control that is not persistently exciting (constant) makes the control term
/// unidentifiable — an honest, documented limit. The recovery of the physical
/// state coefficients still holds, but the constant control collapses into the
/// constant library term rather than the `u` term.
#[test]
fn constant_control_is_not_identifiable() {
    // Build an oscillator forced by a CONSTANT control u == 0.4.
    let dt = 0.005;
    let steps = 4000usize;
    let mut time = Vec::new();
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut state = [1.0, 0.0];
    let mut t = 0.0;
    let forcing = 0.4;
    let rhs = |s: [f64; 2]| [s[1], -OSCILLATOR_K * s[0] - OSCILLATOR_C * s[1] + forcing];
    for _ in 0..=steps {
        time.push(t);
        xs.push(state[0]);
        ys.push(state[1]);
        let k1 = rhs(state);
        let s2 = [state[0] + 0.5 * dt * k1[0], state[1] + 0.5 * dt * k1[1]];
        let k2 = rhs(s2);
        let s3 = [state[0] + 0.5 * dt * k2[0], state[1] + 0.5 * dt * k2[1]];
        let k3 = rhs(s3);
        let s4 = [state[0] + dt * k3[0], state[1] + dt * k3[1]];
        let k4 = rhs(s4);
        state = [
            state[0] + dt / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
            state[1] + dt / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
        ];
        t += dt;
    }
    let us = vec![forcing; time.len()];
    let dataset = Dataset::new(
        TimeAxis::new(time).unwrap(),
        [
            NumericColumn::new(id("x"), xs),
            NumericColumn::new(id("y"), ys),
            NumericColumn::new(id("u"), us),
        ],
    )
    .unwrap();

    let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
    let controlled = discover_controlled(&dataset, &spec, &ControlConfig::default()).unwrap();
    let controlled_rss = controlled.equation(&id("y")).unwrap().residual_sum_squares;

    // A constant control is collinear with the constant library term, so it adds
    // no identifiable information: a states-only fit (which keeps the constant
    // term to absorb the steady forcing) is essentially as good. This is the
    // exact opposite of `control_term_is_necessary` and demonstrates the
    // documented "control must be persistently exciting" limit.
    let time = dataset.time().values();
    let y_values = &dataset.columns()[&id("y")].values;
    let dy = differentiate_series(time, y_values).unwrap();
    let states_only = FeatureLibrary::polynomial([id("x"), id("y")], 2, true).unwrap();
    let matrix = states_only.evaluate(&dataset).unwrap();
    let problem = RegressionProblem::new(matrix.rows, dy).unwrap();
    let states_only_rss = stlsq_standardized(&problem, &ControlConfig::default().sparse)
        .unwrap()
        .residual_sum_squares;

    assert!(
        states_only_rss < 2.0 * controlled_rss + 1e-9,
        "with a constant control the control term should be unnecessary, but states-only RSS {states_only_rss} >> controlled RSS {controlled_rss}"
    );
}
