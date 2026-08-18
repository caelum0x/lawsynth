//! Shared deterministic fixtures for the controlled-discovery integration tests.
//!
//! Everything here is a pure function of fixed constants — no wall clock, no
//! RNG seeded from the environment — so the generated trajectories are
//! bit-identical across runs and machines.
//!
//! Each integration-test binary that includes this module uses a different
//! subset of the helpers, so unused-symbol warnings are expected and allowed.
#![allow(dead_code)]

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// True stiffness of the forced linear oscillator `ẏ = -k·x - c·y + u`.
pub const OSCILLATOR_K: f64 = 2.0;
/// True damping of the forced linear oscillator.
pub const OSCILLATOR_C: f64 = 0.3;
/// True control gain (the coefficient of `u` in the `ẏ` equation).
pub const OSCILLATOR_CONTROL_GAIN: f64 = 1.0;

/// A fixed, persistently-exciting deterministic multi-sine control signal.
///
/// The three incommensurate frequencies and the phase offset make `u(t)` rich
/// enough to identify the control term. It is an explicit function of time, so
/// it can be evaluated at the RK4 intermediate stages for a near-exact
/// integration.
pub fn control_signal(t: f64) -> f64 {
    0.5 * (1.7 * t).sin() + 0.3 * (0.9 * t + 0.5).sin() + 0.2 * (2.3 * t).sin()
}

/// Right-hand side of the forced linear oscillator `ẋ = y`, `ẏ = -k·x - c·y + u(t)`.
fn oscillator_rhs(t: f64, state: [f64; 2]) -> [f64; 2] {
    let [x, y] = state;
    let dx = y;
    let dy = -OSCILLATOR_K * x - OSCILLATOR_C * y + control_signal(t);
    [dx, dy]
}

/// Integrates the forced oscillator with fixed-step RK4 and returns aligned
/// `(time, x, y, u)` samples.
///
/// `dt` is small enough that the trajectory is effectively exact, so a
/// three-point derivative of the sampled states recovers `ẋ`, `ẏ` to well within
/// the recovery tolerance.
pub fn integrate_oscillator(steps: usize, dt: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut time = Vec::with_capacity(steps + 1);
    let mut xs = Vec::with_capacity(steps + 1);
    let mut ys = Vec::with_capacity(steps + 1);
    let mut us = Vec::with_capacity(steps + 1);

    let mut state = [1.0, 0.0];
    let mut t = 0.0;
    for _ in 0..=steps {
        time.push(t);
        xs.push(state[0]);
        ys.push(state[1]);
        us.push(control_signal(t));

        let k1 = oscillator_rhs(t, state);
        let s2 = [state[0] + 0.5 * dt * k1[0], state[1] + 0.5 * dt * k1[1]];
        let k2 = oscillator_rhs(t + 0.5 * dt, s2);
        let s3 = [state[0] + 0.5 * dt * k2[0], state[1] + 0.5 * dt * k2[1]];
        let k3 = oscillator_rhs(t + 0.5 * dt, s3);
        let s4 = [state[0] + dt * k3[0], state[1] + dt * k3[1]];
        let k4 = oscillator_rhs(t + dt, s4);

        state = [
            state[0] + dt / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
            state[1] + dt / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
        ];
        t += dt;
    }
    (time, xs, ys, us)
}

/// Convenience identifier constructor for tests.
pub fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

/// Builds the standard forced-oscillator dataset with columns `x`, `y`, `u`.
///
/// Uses a fine step so the state derivatives are near-exact. 4000 steps at
/// `dt = 0.005` covers `t ∈ [0, 20]`.
pub fn oscillator_dataset() -> Dataset {
    let (time, xs, ys, us) = integrate_oscillator(4000, 0.005);
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [
            NumericColumn::new(id("x"), xs),
            NumericColumn::new(id("y"), ys),
            NumericColumn::new(id("u"), us),
        ],
    )
    .unwrap()
}

/// The label of the constant library term.
///
/// The polynomial library emits the total-degree-0 monomial first, and the
/// expression printer renders the constant `1.0` in scientific notation, so the
/// constant term is always `library_terms[0]`.
pub fn constant_label(model: &lawsynth_control::ControlledModel) -> String {
    model.library_terms[0].clone()
}

/// Reads a single coefficient out of a fitted state equation by its library label.
pub fn coefficient_for(
    model: &lawsynth_control::ControlledModel,
    state: &str,
    term_label: &str,
) -> f64 {
    let equation = model.equation(&id(state)).expect("state equation present in the model");
    let index =
        model.library_terms.iter().position(|label| label == term_label).unwrap_or_else(|| {
            panic!("library term '{term_label}' not found in {:?}", model.library_terms)
        });
    equation.coefficients[index]
}
