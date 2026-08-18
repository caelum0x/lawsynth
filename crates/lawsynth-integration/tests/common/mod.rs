//! Shared deterministic fixtures for the cross-crate integration tests.
//!
//! Every generator here integrates a KNOWN autonomous system from a fixed
//! initial condition with a fixed-step classical RK4 integrator — no RNG, no
//! wall-clock, no I/O. The produced [`Dataset`] is exactly what a user would
//! feed to `discover`, so the tests downstream drive the real discovery →
//! analysis pipeline on data whose true law we already know.
//!
//! Because the same `(system, IC, dt, steps)` always yields the same samples,
//! the whole pipeline built on top of these fixtures is reproducible bit for
//! bit, which is what the determinism tests assert.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// Convenience constructor for a valid identifier (panics on an invalid name,
/// which only happens on a typo in a test).
pub fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// One RK4 step of the autonomous system `ẋ = f(x)` with fixed step `dt`.
///
/// `f` maps a state slice to its derivative vector; the arithmetic order is
/// fixed so the trajectory is reproducible to the bit.
fn rk4_step<F>(state: &[f64], dt: f64, f: &F) -> Vec<f64>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    let n = state.len();
    let k1 = f(state);
    let s2: Vec<f64> = (0..n).map(|i| state[i] + 0.5 * dt * k1[i]).collect();
    let k2 = f(&s2);
    let s3: Vec<f64> = (0..n).map(|i| state[i] + 0.5 * dt * k2[i]).collect();
    let k3 = f(&s3);
    let s4: Vec<f64> = (0..n).map(|i| state[i] + dt * k3[i]).collect();
    let k4 = f(&s4);
    (0..n).map(|i| state[i] + (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i])).collect()
}

/// Integrates `ẋ = f(x)` from `initial` for `steps` steps of size `dt`,
/// returning the time axis and one value column per state dimension (including
/// the initial sample), all row-aligned.
fn integrate<F>(initial: &[f64], dt: f64, steps: usize, f: F) -> (Vec<f64>, Vec<Vec<f64>>)
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    let n = initial.len();
    let mut time = Vec::with_capacity(steps + 1);
    let mut columns: Vec<Vec<f64>> = vec![Vec::with_capacity(steps + 1); n];
    let mut state = initial.to_vec();
    for (dimension, column) in columns.iter_mut().enumerate() {
        column.push(state[dimension]);
    }
    time.push(0.0);
    for step in 0..steps {
        state = rk4_step(&state, dt, &f);
        time.push((step + 1) as f64 * dt);
        for (dimension, column) in columns.iter_mut().enumerate() {
            column.push(state[dimension]);
        }
    }
    (time, columns)
}

/// Assembles a two-state dataset from integrated `x` and `y` columns.
fn dataset_xy(time: Vec<f64>, x: Vec<f64>, y: Vec<f64>) -> Dataset {
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [NumericColumn::new(id("x"), x), NumericColumn::new(id("y"), y)],
    )
    .unwrap()
}

/// Damped linear oscillator `ẋ = y, ẏ = −x − 0.3 y`.
///
/// The origin is a stable spiral (the Jacobian `[[0,1],[-1,-0.3]]` has complex
/// eigenvalues with real part `−0.15`), and the trace is exactly `−0.3` — the
/// value the Lyapunov spectrum's sum must recover. Integrated from `(1, 0)`.
pub fn damped_oscillator_dataset() -> Dataset {
    let (time, cols) = integrate(&[1.0, 0.0], 0.01, 2000, |s| vec![s[1], -s[0] - 0.3 * s[1]]);
    let [x, y]: [Vec<f64>; 2] = cols.try_into().unwrap();
    dataset_xy(time, x, y)
}

/// Undamped harmonic oscillator `ẋ = y, ẏ = −x`.
///
/// The origin is a center (purely imaginary eigenvalues `±i`), the flow is
/// conservative with energy `H = x² + y²`, and every Lyapunov exponent is zero.
/// Integrated from `(1, 0)`, so the true orbit is the unit circle.
pub fn harmonic_oscillator_dataset() -> Dataset {
    let (time, cols) = integrate(&[1.0, 0.0], 0.01, 2000, |s| vec![s[1], -s[0]]);
    let [x, y]: [Vec<f64>; 2] = cols.try_into().unwrap();
    dataset_xy(time, x, y)
}
