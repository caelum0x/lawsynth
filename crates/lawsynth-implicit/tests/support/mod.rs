//! Deterministic data generators shared across the integration tests.
//!
//! Trajectories are produced by a fixed-step classical RK4 integrator so the
//! samples are reproducible bit-for-bit and free of any external dependency.
//!
//! Each integration-test binary links this module independently, so helpers
//! unused by a given binary would otherwise warn.
#![allow(dead_code)]

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// Integrates `dx/dt = f(x)` with fixed-step RK4 and returns `(time, x)`.
pub fn integrate(f: impl Fn(f64) -> f64, x0: f64, dt: f64, steps: usize) -> (Vec<f64>, Vec<f64>) {
    let mut time = Vec::with_capacity(steps + 1);
    let mut xs = Vec::with_capacity(steps + 1);
    let mut x = x0;
    for step in 0..=steps {
        time.push(step as f64 * dt);
        xs.push(x);
        let k1 = f(x);
        let k2 = f(x + 0.5 * dt * k1);
        let k3 = f(x + 0.5 * dt * k2);
        let k4 = f(x + dt * k3);
        x += dt / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
    }
    (time, xs)
}

/// Builds a single-state dataset named `x` from a trajectory.
pub fn dataset_x(time: Vec<f64>, xs: Vec<f64>) -> Dataset {
    let x = Identifier::new("x").unwrap();
    Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(x, xs)]).unwrap()
}

/// Michaelis-Menten decay `ẋ = -Vmax·x / (Km + x)`.
pub fn michaelis_menten(vmax: f64, km: f64) -> impl Fn(f64) -> f64 {
    move |x: f64| -vmax * x / (km + x)
}

/// Linear decay `ẋ = -k·x`.
pub fn linear_decay(k: f64) -> impl Fn(f64) -> f64 {
    move |x: f64| -k * x
}

/// Looks up a numerator/denominator coefficient by term name, defaulting to 0.
pub fn coefficient(terms: &[lawsynth_implicit::MonomialTerm], name: &str) -> f64 {
    terms.iter().find(|term| term.name == name).map(|term| term.coefficient).unwrap_or(0.0)
}
