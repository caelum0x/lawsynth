//! Shared deterministic Euler–Maruyama fixtures for the SDE discovery tests.
//!
//! These helpers generate LONG, seeded sample paths of known SDEs so the tests
//! can check that `discover_sde` recovers the drift and diffusion within a stated
//! statistical tolerance. The RNG is the project's own SplitMix64
//! ([`lawsynth_core::DeterministicRng`]) with a fixed [`lawsynth_core::Seed`];
//! randomness is NEVER drawn from the wall clock, so every path is
//! bit-reproducible.
// Shared across several test binaries; not every helper is used by every one.
#![allow(dead_code)]

use lawsynth_core::{DeterministicRng, Identifier, Seed};
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// One standard-normal sample via Box–Muller over the project SplitMix64 stream.
///
/// This mirrors `lawsynth-sim::euler_maruyama`'s own `standard_normal` so the
/// fixture reproduces the reference integrator exactly, without taking a
/// dependency on that crate.
fn standard_normal(rng: &mut DeterministicRng) -> f64 {
    let u1 = rng.next_f64().max(f64::MIN_POSITIVE);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Integrates a scalar Itô SDE `dX = drift(x) dt + diffusion(x) dW` with the
/// Euler–Maruyama scheme on a regular grid, returning `(time, values)`.
///
/// `steps` transitions produce `steps + 1` samples. The path is deterministic
/// for a given `seed`.
pub fn euler_maruyama_scalar<D, S>(
    initial: f64,
    dt: f64,
    steps: usize,
    seed: u64,
    drift: D,
    diffusion: S,
) -> (Vec<f64>, Vec<f64>)
where
    D: Fn(f64) -> f64,
    S: Fn(f64) -> f64,
{
    let mut rng = Seed::new(seed).rng();
    let root_dt = dt.sqrt();
    let mut x = initial;
    let mut time = Vec::with_capacity(steps + 1);
    let mut values = Vec::with_capacity(steps + 1);
    time.push(0.0);
    values.push(x);
    for step in 0..steps {
        x += drift(x) * dt + diffusion(x) * root_dt * standard_normal(&mut rng);
        time.push((step + 1) as f64 * dt);
        values.push(x);
    }
    (time, values)
}

/// Wraps a `(time, values)` path into a single-column `Dataset` named `x`.
pub fn scalar_dataset(time: Vec<f64>, values: Vec<f64>) -> Dataset {
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [NumericColumn::new(Identifier::new("x").unwrap(), values)],
    )
    .unwrap()
}

/// An Ornstein–Uhlenbeck path `dX = −θ X dt + σ dW` (constant diffusion `σ²`).
pub fn ornstein_uhlenbeck(
    theta: f64,
    sigma: f64,
    dt: f64,
    steps: usize,
    seed: u64,
) -> (Vec<f64>, Vec<f64>) {
    euler_maruyama_scalar(0.0, dt, steps, seed, |x| -theta * x, move |_| sigma)
}

/// A geometric-Brownian-motion path `dX = μ X dt + σ X dW` (diffusion `σ² x²`).
pub fn geometric_brownian_motion(
    mu: f64,
    sigma: f64,
    initial: f64,
    dt: f64,
    steps: usize,
    seed: u64,
) -> (Vec<f64>, Vec<f64>) {
    euler_maruyama_scalar(initial, dt, steps, seed, move |x| mu * x, move |x| sigma * x)
}

/// An ensemble of `trajectories` geometric-Brownian-motion paths, concatenated
/// into a single continuous-time series for multi-trajectory discovery.
///
/// GBM has no stationary distribution and a single long path either explodes or
/// collapses, so an ensemble of short paths launched from initial conditions
/// spread across `[x_low, x_high]` is the honest way to sample the state space
/// with good occupancy AND a detectable drift. Returns `(time, values)` with
/// `trajectories * segment_len` rows on a uniform global grid; pair it with
/// `SdeConfig::with_trajectories(trajectories)` so the boundary increments are
/// skipped.
pub fn geometric_brownian_ensemble(
    mu: f64,
    sigma: f64,
    x_low: f64,
    x_high: f64,
    dt: f64,
    trajectories: usize,
    segment_len: usize,
    seed: u64,
) -> (Vec<f64>, Vec<f64>) {
    let steps = segment_len - 1;
    let mut time = Vec::with_capacity(trajectories * segment_len);
    let mut values = Vec::with_capacity(trajectories * segment_len);
    let mut clock = 0usize;
    for k in 0..trajectories {
        let fraction = if trajectories > 1 { k as f64 / (trajectories - 1) as f64 } else { 0.0 };
        let x0 = x_low + (x_high - x_low) * fraction;
        let (_, path) = euler_maruyama_scalar(
            x0,
            dt,
            steps,
            seed.wrapping_add(k as u64),
            move |x| mu * x,
            move |x| sigma * x,
        );
        for value in path {
            time.push(clock as f64 * dt);
            values.push(value);
            clock += 1;
        }
    }
    (time, values)
}

/// A double-well path `dX = (X − X³) dt + σ dW` (nonlinear cubic drift).
pub fn double_well(sigma: f64, dt: f64, steps: usize, seed: u64) -> (Vec<f64>, Vec<f64>) {
    euler_maruyama_scalar(0.0, dt, steps, seed, |x| x - x * x * x, move |_| sigma)
}

/// The identifier for the single state column `x`.
pub fn x() -> Identifier {
    Identifier::new("x").unwrap()
}

/// Flattens every `f64` in a model into its raw bit pattern, for bit-identical
/// determinism assertions (`f64::to_bits`).
pub fn model_bits(model: &lawsynth_sde::SdeModel) -> Vec<u64> {
    let mut bits = vec![model.dt.to_bits(), model.increment_count as u64];
    for state in &model.states {
        bits.push(state.trusted_bins as u64);
        for bin in &state.bins {
            bits.push(bin.x_center.to_bits());
            bits.push(bin.drift.to_bits());
            bits.push(bin.diffusion.to_bits());
            bits.push(bin.count as u64);
        }
        for law in [&state.drift_law, &state.diffusion_law] {
            bits.push(law.residual_sum_squares.to_bits());
            for term in &law.terms {
                bits.push(u64::from(term.power));
                bits.push(term.coefficient.to_bits());
            }
        }
    }
    bits
}
