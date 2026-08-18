//! Deterministic estimator simulation: run the plant and the observer in
//! lockstep and record the shrinking estimation error.
//!
//! The coupled plant–observer system
//!
//! ```text
//! ẋ  = A x  + B u
//! x̂̇ = A x̂ + B u + L (y − C x̂),   y = C x + noise
//! ```
//!
//! is integrated as **one** augmented ODE with a fixed-step RK4 from a possibly
//! wrong initial estimate. Integrating the pair together (rather than freezing
//! the measurement across a step) keeps the continuous error dynamics
//! `ė = (A − L C) e` exact up to RK4 truncation, so a well-designed `L` drives
//! `‖x − x̂‖ → 0`; this simulation is what demonstrates that convergence
//! numerically.
//!
//! The control input `u` is a zero-order hold over each step (`u_k` constant on
//! `[t_k, t_{k+1})`). Measurement noise, when requested, is drawn once per step
//! from the seeded generator and likewise held over the step, modelling a
//! sampled noisy sensor; the noise-free case (`None`) yields exact convergence.

use lawsynth_koopman::Matrix;

use crate::error::EstimateError;
use crate::linalg;
use crate::noise::{GaussianStream, MeasurementNoise};
use crate::observer::Observer;

/// The paired true / estimated trajectories and the estimation error over time.
///
/// All series have length `steps + 1` (initial condition through the final
/// step). `errors[k] = ‖true_states[k] − estimates[k]‖₂`.
#[derive(Clone, Debug)]
pub struct EstimateTrajectory {
    /// Sample times `t_k = k · dt`.
    pub times: Vec<f64>,
    /// The true plant state `x(t_k)`, each of length `n`.
    pub true_states: Vec<Vec<f64>>,
    /// The observer estimate `x̂(t_k)`, each of length `n`.
    pub estimates: Vec<Vec<f64>>,
    /// The (optionally noisy) measurement `y(t_k)` fed to the observer, each of
    /// length `p`.
    pub measurements: Vec<Vec<f64>>,
    /// The estimation error norm `‖x(t_k) − x̂(t_k)‖₂`.
    pub errors: Vec<f64>,
}

impl EstimateTrajectory {
    /// The estimation error at the final sample.
    pub fn final_error(&self) -> f64 {
        *self.errors.last().unwrap_or(&0.0)
    }

    /// The largest estimation error over the whole horizon.
    pub fn max_error(&self) -> f64 {
        self.errors.iter().copied().fold(0.0_f64, f64::max)
    }

    /// The estimation error at the initial sample.
    pub fn initial_error(&self) -> f64 {
        *self.errors.first().unwrap_or(&0.0)
    }
}

/// Simulates the observer against the plant and returns both trajectories.
///
/// - `observer`: the designed estimator (its gain `L` is `n × p`).
/// - `a`, `b`, `c`: the plant `ẋ = A x + B u`, `y = C x` (`A` is `n × n`,
///   `B` is `n × m`, `C` is `p × n`).
/// - `true_x0`, `est_x0`: the true and estimated initial states (length `n`).
/// - `inputs`: the control signal, one `u_k` (length `m`) per step. An empty
///   slice means the autonomous case `u ≡ 0`; otherwise it must have `steps`
///   entries.
/// - `noise`: optional seeded Gaussian measurement noise; `None` gives the exact
///   measurement `y = C x`.
/// - `dt`, `steps`: the fixed time step and number of steps (`dt > 0`,
///   `steps > 0`).
///
/// # Errors
/// Returns [`EstimateError::ShapeMismatch`] for any inconsistent operand shape,
/// [`EstimateError::NonFiniteValue`] for non-finite matrix data, and
/// [`EstimateError::InvalidTimeStep`] for `dt ≤ 0` or `steps == 0`.
#[allow(clippy::too_many_arguments)]
pub fn run_observer(
    observer: &Observer,
    a: &Matrix,
    b: &Matrix,
    c: &Matrix,
    true_x0: &[f64],
    est_x0: &[f64],
    inputs: &[Vec<f64>],
    noise: Option<MeasurementNoise>,
    dt: f64,
    steps: usize,
) -> Result<EstimateTrajectory, EstimateError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(EstimateError::NonSquare);
    }
    if n == 0 {
        return Err(EstimateError::EmptyMatrix);
    }
    if b.rows() != n {
        return Err(EstimateError::ShapeMismatch);
    }
    let m = b.cols();
    let p = c.rows();
    if c.cols() != n {
        return Err(EstimateError::ShapeMismatch);
    }
    if observer.gain.rows() != n || observer.gain.cols() != p {
        return Err(EstimateError::ShapeMismatch);
    }
    if true_x0.len() != n || est_x0.len() != n {
        return Err(EstimateError::ShapeMismatch);
    }
    if !linalg::is_finite(a)
        || !linalg::is_finite(b)
        || !linalg::is_finite(c)
        || !linalg::is_finite(&observer.gain)
    {
        return Err(EstimateError::NonFiniteValue);
    }
    if dt.is_nan() || dt <= 0.0 || steps == 0 {
        return Err(EstimateError::InvalidTimeStep);
    }
    if !inputs.is_empty() {
        if inputs.len() != steps {
            return Err(EstimateError::ShapeMismatch);
        }
        if inputs.iter().any(|u| u.len() != m) {
            return Err(EstimateError::ShapeMismatch);
        }
    }

    let l = &observer.gain;
    let mut stream = noise.map(GaussianStream::new);

    let mut times = Vec::with_capacity(steps + 1);
    let mut true_states = Vec::with_capacity(steps + 1);
    let mut estimates = Vec::with_capacity(steps + 1);
    let mut measurements = Vec::with_capacity(steps + 1);
    let mut errors = Vec::with_capacity(steps + 1);

    let mut x_true = true_x0.to_vec();
    let mut x_hat = est_x0.to_vec();

    for k in 0..=steps {
        // Measurement-noise sample for this step, held over `[t_k, t_{k+1})`.
        let noise_k = match stream.as_mut() {
            Some(stream) => stream.sample(p),
            None => vec![0.0; p],
        };

        // The recorded (optionally noisy) measurement at this instant.
        let mut y = linalg::mat_vec(c, &x_true)?;
        for (yi, ni) in y.iter_mut().zip(&noise_k) {
            *yi += ni;
        }

        times.push(k as f64 * dt);
        errors.push(error_norm(&x_true, &x_hat));
        true_states.push(x_true.clone());
        estimates.push(x_hat.clone());
        measurements.push(y);

        if k == steps {
            break;
        }

        // Control input for this step (zero-order hold); zeros if autonomous.
        let u = if inputs.is_empty() { vec![0.0; m] } else { inputs[k].clone() };
        let bu = if m == 0 { vec![0.0; n] } else { linalg::mat_vec(b, &u)? };

        // Advance the coupled (plant, observer) system together so the error
        // dynamics stay exact within the step (the plant's instantaneous output
        // drives the observer, plus the step's held noise).
        let derivative =
            |xt: &[f64], xh: &[f64]| coupled_derivative(a, l, c, &bu, &noise_k, xt, xh);
        (x_true, x_hat) = rk4_step(dt, &x_true, &x_hat, derivative)?;
    }

    Ok(EstimateTrajectory { times, true_states, estimates, measurements, errors })
}

/// The coupled derivative `(ẋ, x̂̇)` for the augmented plant–observer system,
/// with `B u` and the step's measurement noise held constant.
fn coupled_derivative(
    a: &Matrix,
    l: &Matrix,
    c: &Matrix,
    bu: &[f64],
    noise: &[f64],
    x_true: &[f64],
    x_hat: &[f64],
) -> Result<(Vec<f64>, Vec<f64>), EstimateError> {
    // Plant: ẋ = A x + B u.
    let ax = linalg::mat_vec(a, x_true)?;
    let dx: Vec<f64> = ax.iter().zip(bu).map(|(ai, bi)| ai + bi).collect();

    // Observer: x̂̇ = A x̂ + B u + L (y − C x̂), with y = C x + noise instantaneous.
    let cx = linalg::mat_vec(c, x_true)?;
    let cxhat = linalg::mat_vec(c, x_hat)?;
    let innovation: Vec<f64> = (0..cx.len()).map(|i| cx[i] + noise[i] - cxhat[i]).collect();
    let correction = linalg::mat_vec(l, &innovation)?;
    let axhat = linalg::mat_vec(a, x_hat)?;
    let dxhat: Vec<f64> = (0..axhat.len()).map(|i| axhat[i] + bu[i] + correction[i]).collect();

    Ok((dx, dxhat))
}

/// One fixed-step classical RK4 step for the coupled system `(ẋ, x̂̇) = f(x, x̂)`
/// with `f` autonomous over the step (inputs and step noise held constant).
fn rk4_step<F>(
    dt: f64,
    x: &[f64],
    x_hat: &[f64],
    f: F,
) -> Result<(Vec<f64>, Vec<f64>), EstimateError>
where
    F: Fn(&[f64], &[f64]) -> Result<(Vec<f64>, Vec<f64>), EstimateError>,
{
    let advance = |base: &[f64], slope: &[f64], h: f64| -> Vec<f64> {
        base.iter().zip(slope).map(|(bi, si)| bi + h * si).collect()
    };

    let (k1x, k1h) = f(x, x_hat)?;
    let (k2x, k2h) = f(&advance(x, &k1x, 0.5 * dt), &advance(x_hat, &k1h, 0.5 * dt))?;
    let (k3x, k3h) = f(&advance(x, &k2x, 0.5 * dt), &advance(x_hat, &k2h, 0.5 * dt))?;
    let (k4x, k4h) = f(&advance(x, &k3x, dt), &advance(x_hat, &k3h, dt))?;

    let combine = |base: &[f64], a: &[f64], b: &[f64], c: &[f64], d: &[f64]| -> Vec<f64> {
        (0..base.len())
            .map(|i| base[i] + dt / 6.0 * (a[i] + 2.0 * b[i] + 2.0 * c[i] + d[i]))
            .collect()
    };

    let next_x = combine(x, &k1x, &k2x, &k3x, &k4x);
    let next_hat = combine(x_hat, &k1h, &k2h, &k3h, &k4h);
    Ok((next_x, next_hat))
}

/// The Euclidean estimation-error norm `‖x − x̂‖₂`.
fn error_norm(x_true: &[f64], x_hat: &[f64]) -> f64 {
    let diff: Vec<f64> = x_true.iter().zip(x_hat).map(|(a, b)| a - b).collect();
    linalg::norm2(&diff)
}
