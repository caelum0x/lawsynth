//! The successive-linearization MPC loop: relinearize, design a local LQR gain,
//! apply the first (saturated) move, and RK4-advance the true nonlinear plant.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_feedback::lqr;
use lawsynth_koopman::Matrix;

use crate::config::MpcConfig;
use crate::error::MpcError;
use crate::model::ControlModel;
use crate::trajectory::MpcTrajectory;

/// Drives a discovered nonlinear model `ẋ = f(x, u)` toward a setpoint by
/// successive-linearization (gain-scheduled LQR) model-predictive control.
///
/// At each of `config.steps` control steps, with current state `x`:
///
/// 1. **Linearize** about `(x, u_ref)`: `A = ∂f/∂x` from the analytic Jacobian
///    and `B = ∂f/∂u` from the symbolic control partials, both evaluated at the
///    point.
/// 2. **Design** the infinite-horizon LQR gain `K` for the local pair
///    `(A, B)` with weights `Q, R` (via `lawsynth-feedback`).
/// 3. **Apply** the first move `u = clamp(u_ref − K (x − x_ref), u_min, u_max)`.
/// 4. **Advance** the true nonlinear plant one fixed step `dt` by RK4 with `u`
///    held constant.
///
/// The closed-loop state and control trajectory is returned. See the boundary
/// spec `specs/model-predictive-control/README.md` for the contract and its
/// honest limits — this is *successive-linearization LQR-MPC*, not a constrained
/// QP-MPC with horizon optimization.
///
/// # Errors
///
/// Boundary violations (empty state/control, mis-sized weights or setpoint,
/// non-finite configuration, non-positive `dt`, zero horizon, inconsistent
/// saturation) return the matching [`MpcError`] before any integration. During
/// the loop, a failed linearization, LQR design (e.g. `R` not positive definite,
/// an unstabilizable point), or plant evaluation is propagated as a typed error.
pub fn mpc_control(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    controls: &[Identifier],
    config: &MpcConfig,
) -> Result<MpcTrajectory, MpcError> {
    let model = ControlModel::build(fields, states, controls)?;
    let n = model.state_dim();
    let m = model.control_dim();

    validate(config, n, m)?;

    let mut state = config.initial_state.clone();
    let mut recorded_states = Vec::with_capacity(config.steps + 1);
    let mut recorded_controls = Vec::with_capacity(config.steps);
    let mut recorded_gains = Vec::with_capacity(config.steps);
    let mut times = Vec::with_capacity(config.steps + 1);

    recorded_states.push(state.clone());
    times.push(0.0);

    for step in 0..config.steps {
        // (1) Linearize about the current state at the control reference.
        let a = model.state_matrix(&state, &config.control_reference)?;
        let b = model.control_matrix(&state, &config.control_reference)?;

        // (2) Local optimal gain for this linearization.
        let gain = lqr(&a, &b, &config.state_weight, &config.control_weight)?;

        // (3) First move u = clamp(u_ref − K (x − x_ref), u_min, u_max).
        let control = feedback_move(&gain.k, &state, config)?;

        // (4) Advance the true nonlinear plant one fixed step with u held.
        state = rk4_step(&model, &state, &control, config.dt)?;

        recorded_controls.push(control);
        recorded_gains.push(gain.k);
        recorded_states.push(state.clone());
        times.push(config.dt * (step + 1) as f64);
    }

    Ok(MpcTrajectory::new(recorded_states, recorded_controls, recorded_gains, times))
}

/// Computes the saturated feedback move `clamp(u_ref − K (x − x_ref), lo, hi)`.
fn feedback_move(gain: &Matrix, state: &[f64], config: &MpcConfig) -> Result<Vec<f64>, MpcError> {
    let error: Vec<f64> =
        state.iter().zip(&config.setpoint).map(|(value, target)| value - target).collect();
    // K is m×n; K·(x − x_ref) is the length-m correction.
    let correction = gain.mat_vec(&error).map_err(MpcError::from)?;

    let control = config
        .control_reference
        .iter()
        .zip(&correction)
        .enumerate()
        .map(|(index, (reference, delta))| {
            let raw = reference - delta;
            clamp(raw, config, index)
        })
        .collect();
    Ok(control)
}

/// Clamps a single control channel to its saturation bounds, if any.
fn clamp(value: f64, config: &MpcConfig, index: usize) -> f64 {
    let mut clamped = value;
    if let Some(min) = config.control_min.as_ref().map(|bounds| bounds[index]) {
        if clamped < min {
            clamped = min;
        }
    }
    if let Some(max) = config.control_max.as_ref().map(|bounds| bounds[index]) {
        if clamped > max {
            clamped = max;
        }
    }
    clamped
}

/// One fixed-step classical RK4 advance of `ẋ = f(x, u)` with `u` held constant.
fn rk4_step(
    model: &ControlModel,
    state: &[f64],
    control: &[f64],
    dt: f64,
) -> Result<Vec<f64>, MpcError> {
    let k1 = model.field(state, control)?;
    let k2 = model.field(&axpy(state, &k1, dt / 2.0), control)?;
    let k3 = model.field(&axpy(state, &k2, dt / 2.0), control)?;
    let k4 = model.field(&axpy(state, &k3, dt), control)?;

    let next = state
        .iter()
        .enumerate()
        .map(|(i, value)| value + (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]))
        .collect();
    Ok(next)
}

/// Returns `base + scale · delta`, element-wise (the RK4 stage points).
fn axpy(base: &[f64], delta: &[f64], scale: f64) -> Vec<f64> {
    base.iter().zip(delta).map(|(value, slope)| value + scale * slope).collect()
}

/// Validates the configuration against the model dimensions `n` (states) and
/// `m` (controls) before any integration begins.
fn validate(config: &MpcConfig, n: usize, m: usize) -> Result<(), MpcError> {
    check_len("initial_state", config.initial_state.len(), n)?;
    check_len("setpoint", config.setpoint.len(), n)?;
    check_len("control_reference", config.control_reference.len(), m)?;
    check_square("state_weight", &config.state_weight, n)?;
    check_square("control_weight", &config.control_weight, m)?;

    if !config.dt.is_finite() || config.dt <= 0.0 {
        return Err(MpcError::InvalidTimeStep(config.dt));
    }
    if config.steps == 0 {
        return Err(MpcError::EmptyHorizon);
    }

    check_finite("initial_state", &config.initial_state)?;
    check_finite("setpoint", &config.setpoint)?;
    check_finite("control_reference", &config.control_reference)?;

    validate_saturation(config, m)?;
    Ok(())
}

/// Validates the optional saturation bounds: correct length, finite, and
/// `min ≤ max` per channel.
fn validate_saturation(config: &MpcConfig, m: usize) -> Result<(), MpcError> {
    if let Some(min) = &config.control_min {
        check_len("control_min", min.len(), m)?;
        check_finite("control_min", min)?;
    }
    if let Some(max) = &config.control_max {
        check_len("control_max", max.len(), m)?;
        check_finite("control_max", max)?;
    }
    if let (Some(min), Some(max)) = (&config.control_min, &config.control_max) {
        for (index, (lo, hi)) in min.iter().zip(max).enumerate() {
            if lo > hi {
                return Err(MpcError::InvalidSaturation { index });
            }
        }
    }
    Ok(())
}

fn check_len(what: &'static str, actual: usize, expected: usize) -> Result<(), MpcError> {
    if actual != expected {
        return Err(MpcError::DimensionMismatch { what, expected, actual });
    }
    Ok(())
}

fn check_square(what: &'static str, matrix: &Matrix, expected: usize) -> Result<(), MpcError> {
    if matrix.rows() != expected || matrix.cols() != expected {
        // Report the row count as the observed dimension; a rectangular weight is
        // as much a dimension error as a wrong-order square one.
        return Err(MpcError::DimensionMismatch { what, expected, actual: matrix.rows() });
    }
    Ok(())
}

fn check_finite(what: &'static str, values: &[f64]) -> Result<(), MpcError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(MpcError::NonFiniteConfig(what));
    }
    Ok(())
}
