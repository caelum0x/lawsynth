use crate::SimulationError;
use lawsynth_core::Seed;

/// Euler-Maruyama configuration for a vector stochastic differential equation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdeConfig {
    pub start: f64,
    pub end: f64,
    pub step: f64,
    pub seed: Seed,
}
/// Simulated SDE samples including the initial state.
#[derive(Clone, Debug, PartialEq)]
pub struct SdeTrajectory {
    pub time: Vec<f64>,
    pub values: Vec<Vec<f64>>,
}

/// Integrates diagonal-noise SDEs `dx = drift dt + diffusion dW` deterministically from a seed.
pub fn euler_maruyama<F, G>(
    initial: &[f64],
    config: SdeConfig,
    drift: F,
    diffusion: G,
) -> Result<SdeTrajectory, SimulationError>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
    G: Fn(f64, &[f64]) -> Vec<f64>,
{
    if initial.is_empty()
        || initial.iter().any(|value| !value.is_finite())
        || !config.start.is_finite()
        || !config.end.is_finite()
        || !config.step.is_finite()
        || config.end <= config.start
        || config.step <= 0.0
    {
        return Err(SimulationError::InvalidTimeGrid);
    }
    let mut rng = config.seed.rng();
    let mut time = config.start;
    let mut state = initial.to_vec();
    let mut trajectory = SdeTrajectory {
        time: vec![time],
        values: vec![state.clone()],
    };
    while time < config.end {
        let dt = (config.end - time).min(config.step);
        let drift = drift(time, &state);
        let diffusion = diffusion(time, &state);
        if drift.len() != state.len() || diffusion.len() != state.len() {
            return Err(SimulationError::InvalidTimeGrid);
        }
        let root_dt = dt.sqrt();
        for index in 0..state.len() {
            state[index] +=
                drift[index] * dt + diffusion[index] * root_dt * standard_normal(&mut rng);
            if !state[index].is_finite() {
                return Err(SimulationError::InvalidTimeGrid);
            }
        }
        let next_time = time + dt;
        if next_time <= time {
            return Err(SimulationError::TimeResolutionLoss);
        }
        time = next_time;
        trajectory.time.push(time);
        trajectory.values.push(state.clone());
    }
    Ok(trajectory)
}
fn standard_normal(rng: &mut lawsynth_core::DeterministicRng) -> f64 {
    let u1 = rng.next_f64().max(f64::MIN_POSITIVE);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}
