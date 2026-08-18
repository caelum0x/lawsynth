//! Shared deterministic data fixtures for the model-selection integration tests.
//!
//! Every trajectory is produced by a fixed-step classical RK4 integrator with no
//! randomness, so the datasets — and therefore every downstream selection report
//! — are bit-reproducible run to run.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// Builds an identifier from a static name.
pub fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// One RK4 step of a 2-D autonomous system `f(x, v) -> (dx, dv)`.
fn rk4_step_2d(f: impl Fn(f64, f64) -> (f64, f64), x: f64, v: f64, dt: f64) -> (f64, f64) {
    let (k1x, k1v) = f(x, v);
    let (k2x, k2v) = f(x + dt * k1x / 2.0, v + dt * k1v / 2.0);
    let (k3x, k3v) = f(x + dt * k2x / 2.0, v + dt * k2v / 2.0);
    let (k4x, k4v) = f(x + dt * k3x, v + dt * k3v);
    (
        x + dt * (k1x + 2.0 * k2x + 2.0 * k3x + k4x) / 6.0,
        v + dt * (k1v + 2.0 * k2v + 2.0 * k3v + k4v) / 6.0,
    )
}

/// One RK4 step of a scalar autonomous system `f(x) -> dx`.
fn rk4_step_1d(f: impl Fn(f64) -> f64, x: f64, dt: f64) -> f64 {
    let k1 = f(x);
    let k2 = f(x + dt * k1 / 2.0);
    let k3 = f(x + dt * k2 / 2.0);
    let k4 = f(x + dt * k3);
    x + dt * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0
}

/// Integrates a 2-D system into an aligned `(x, v)` dataset.
fn integrate_2d(
    f: impl Fn(f64, f64) -> (f64, f64),
    x0: f64,
    v0: f64,
    dt: f64,
    steps: usize,
) -> Dataset {
    let mut xs = vec![x0];
    let mut vs = vec![v0];
    for _ in 0..steps {
        let (nx, nv) = rk4_step_2d(&f, *xs.last().unwrap(), *vs.last().unwrap(), dt);
        xs.push(nx);
        vs.push(nv);
    }
    let time = (0..xs.len()).map(|k| k as f64 * dt).collect::<Vec<_>>();
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [NumericColumn::new(id("x"), xs), NumericColumn::new(id("v"), vs)],
    )
    .unwrap()
}

/// A **linear** damped oscillator (true polynomial degree 1):
/// `dx/dt = v`, `dv/dt = -0.4 v - x`. Oscillatory and decaying, so every fold
/// segment carries variance to score against.
pub fn linear_oscillator() -> Dataset {
    integrate_2d(|x, v| (v, -0.4 * v - x), 1.5, 0.0, 0.02, 600)
}

/// An **unforced damped cubic (Duffing)** oscillator (true polynomial degree 3):
/// `dx/dt = v`, `dv/dt = -0.3 v - x - x^3`. The cubic restoring term is
/// significant at the chosen amplitude, so a degree-1 or degree-2 library cannot
/// represent it. Single-well and damped, hence non-chaotic and simulatable.
pub fn cubic_oscillator() -> Dataset {
    integrate_2d(|x, v| (v, -0.3 * v - x - x.powi(3)), 1.5, 0.0, 0.02, 600)
}

/// A scalar **logistic** system (true polynomial degree 2): `dx/dt = x - x^2`.
/// The saturating transient is nonlinear, so degree 1 underfits while degree 2
/// recovers it.
pub fn logistic() -> Dataset {
    let dt = 0.02;
    let steps = 300;
    let mut xs = vec![0.15];
    for _ in 0..steps {
        xs.push(rk4_step_1d(|x| x - x * x, *xs.last().unwrap(), dt));
    }
    let time = (0..xs.len()).map(|k| k as f64 * dt).collect::<Vec<_>>();
    Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(id("x"), xs)]).unwrap()
}
