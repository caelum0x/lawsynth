//! Shared deterministic fixtures for the weak-form integration tests.
// Shared across multiple test binaries; not every helper is used by every one.
#![allow(dead_code)]

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// A damped linear oscillator with the ground-truth dynamics
///
/// ```text
/// ẋ = y
/// ẏ = -x - 0.3 y
/// ```
///
/// integrated with fine RK4 so the sampled trajectory is, for practical
/// purposes, the exact solution. Over a polynomial library of degree `d >= 1`
/// with a constant term `[1, x, y, …]`, the true coefficient rows are
/// `x' = [0, 0, 1, …]` and `y' = [0, -1, -0.3, …]`.
pub const DAMPING: f64 = 0.3;

/// Right-hand side of the oscillator.
fn rhs(state: [f64; 2]) -> [f64; 2] {
    let [x, y] = state;
    [y, -x - DAMPING * y]
}

fn rk4_step(state: [f64; 2], dt: f64) -> [f64; 2] {
    let k1 = rhs(state);
    let k2 = rhs([state[0] + 0.5 * dt * k1[0], state[1] + 0.5 * dt * k1[1]]);
    let k3 = rhs([state[0] + 0.5 * dt * k2[0], state[1] + 0.5 * dt * k2[1]]);
    let k4 = rhs([state[0] + dt * k3[0], state[1] + dt * k3[1]]);
    [
        state[0] + dt / 6.0 * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]),
        state[1] + dt / 6.0 * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]),
    ]
}

/// Generates the clean oscillator trajectory as (time, x, y) columns.
pub fn oscillator(samples: usize, dt: f64) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut state = [1.0, 0.0];
    let mut time = Vec::with_capacity(samples);
    let mut xs = Vec::with_capacity(samples);
    let mut ys = Vec::with_capacity(samples);
    for index in 0..samples {
        time.push(index as f64 * dt);
        xs.push(state[0]);
        ys.push(state[1]);
        state = rk4_step(state, dt);
    }
    (time, xs, ys)
}

/// Builds a `Dataset` with lexicographic columns `x` then `y`.
pub fn dataset(time: Vec<f64>, xs: Vec<f64>, ys: Vec<f64>) -> Dataset {
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [
            NumericColumn::new(Identifier::new("x").unwrap(), xs),
            NumericColumn::new(Identifier::new("y").unwrap(), ys),
        ],
    )
    .unwrap()
}

/// A tiny deterministic LCG returning standard-normal noise via Box–Muller.
///
/// Seeded from a fixed constant so the noisy dataset is bit-reproducible: the
/// tests never draw randomness from the wall clock.
pub struct Noise {
    state: u64,
}

impl Noise {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_uniform(&mut self) -> f64 {
        // Numerical Recipes LCG constants.
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        // Use the top 53 bits for a uniform in (0, 1).
        let bits = self.state >> 11;
        (bits as f64 + 0.5) / (1u64 << 53) as f64
    }

    /// One standard-normal sample.
    pub fn normal(&mut self) -> f64 {
        let u1 = self.next_uniform();
        let u2 = self.next_uniform();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// Adds zero-mean Gaussian noise of standard deviation `sigma` to a column.
pub fn add_noise(values: &[f64], sigma: f64, noise: &mut Noise) -> Vec<f64> {
    values.iter().map(|&v| v + sigma * noise.normal()).collect()
}

/// Euclidean distance between a fitted coefficient row and the truth.
pub fn coefficient_error(fitted: &[f64], truth: &[f64]) -> f64 {
    fitted.iter().zip(truth).map(|(a, b)| (a - b) * (a - b)).sum::<f64>().sqrt()
}

/// Central finite-difference derivative on a regular grid — the noise-sensitive
/// step the weak form avoids. Endpoints use one-sided differences.
pub fn central_difference(time: &[f64], values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut derivative = vec![0.0; n];
    derivative[0] = (values[1] - values[0]) / (time[1] - time[0]);
    derivative[n - 1] = (values[n - 1] - values[n - 2]) / (time[n - 1] - time[n - 2]);
    for i in 1..n - 1 {
        derivative[i] = (values[i + 1] - values[i - 1]) / (time[i + 1] - time[i - 1]);
    }
    derivative
}
