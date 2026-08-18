//! Fixed-step RK4 integration of the augmented state-and-sensitivity system.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

use crate::config::SensitivityConfig;
use crate::error::SensitivityError;
use crate::system::AugmentedSystem;
use crate::trajectory::SensitivityTrajectory;

/// Integrates the forward-sensitivity (variational) equations of a discovered
/// model `ẋ = f(x; θ)` and returns the state trajectory together with the
/// trajectory sensitivities `S_j(t) = ∂x(t)/∂θ_j` for every parameter.
///
/// The augmented system `(x, S_1, …, S_p)` — with sensitivity dynamics
/// `Ṡ_j = J_x·S_j + f_{θ_j}` and initial condition `S_j(0) = 0` — is advanced by
/// one shared fixed-step fourth-order Runge–Kutta scheme, so the state and the
/// sensitivities see identical stage points and stay mutually consistent. Both
/// `J_x = ∂f/∂x` and `f_{θ_j} = ∂f/∂θ_j` are analytic: the Jacobian comes from
/// `lawsynth-jacobian` and the parameter partials from symbolic differentiation
/// of the fields. No finite differencing appears anywhere in the integrator.
///
/// # Arguments
///
/// - `fields`: the discovered vector field, one `(state, f_i)` pair per state.
/// - `states`: the state ordering that indexes every output vector.
/// - `parameters`: the discovered coefficients whose sensitivities to compute.
/// - `initial`: the initial state `x(t0)`, in `states` order.
/// - `parameter_values`: the nominal parameter values `θ`, in `parameters` order.
/// - `config`: the start time, step, and step count.
///
/// # Errors
///
/// Returns a typed [`SensitivityError`] for an empty state space, a dimension
/// mismatch, a duplicated parameter, a parameter that is also a state, a field
/// symbol that is neither a state nor a parameter, an invalid config, a Jacobian
/// assembly/differentiation failure, or a numeric evaluation failure during
/// integration.
pub fn forward_sensitivities(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    parameters: &[Identifier],
    initial: &[f64],
    parameter_values: &[f64],
    config: &SensitivityConfig,
) -> Result<SensitivityTrajectory, SensitivityError> {
    config.validate()?;

    if states.is_empty() {
        return Err(SensitivityError::EmptyStateSpace);
    }
    if initial.len() != states.len() {
        return Err(SensitivityError::StateDimensionMismatch {
            states: states.len(),
            initial: initial.len(),
        });
    }
    for (state, value) in states.iter().zip(initial) {
        if !value.is_finite() {
            return Err(SensitivityError::NonFiniteInput { symbol: state.clone(), value: *value });
        }
    }

    let system = AugmentedSystem::assemble(fields, states, parameters, parameter_values)?;
    let n = system.dimension();
    let p = system.parameter_count();

    // Initial augmented state: x(t0) known, every sensitivity block S_j(0) = 0
    // because the initial state does not depend on the parameters.
    let mut y = vec![0.0; system.augmented_len()];
    y[..n].copy_from_slice(initial);

    let sample_count = config.steps() + 1;
    let mut times = Vec::with_capacity(sample_count);
    let mut state_series: Vec<Vec<f64>> = Vec::with_capacity(sample_count);
    let mut sensitivity_series: Vec<Vec<Vec<f64>>> = vec![Vec::with_capacity(sample_count); p];

    record(&y, n, p, &mut state_series, &mut sensitivity_series);
    times.push(config.start());

    let dt = config.step();
    for step in 0..config.steps() {
        y = rk4_step(&system, &y, dt)?;
        // Compute the sample time from the step index rather than accumulating,
        // so rounding does not drift and the grid stays reproducible.
        times.push(config.start() + (step + 1) as f64 * dt);
        record(&y, n, p, &mut state_series, &mut sensitivity_series);
    }

    Ok(SensitivityTrajectory::new(
        states.to_vec(),
        parameters.to_vec(),
        times,
        state_series,
        sensitivity_series,
    ))
}

/// Appends the state and per-parameter sensitivity slices of `y` to the series.
fn record(
    y: &[f64],
    n: usize,
    p: usize,
    state_series: &mut Vec<Vec<f64>>,
    sensitivity_series: &mut [Vec<Vec<f64>>],
) {
    state_series.push(y[..n].to_vec());
    for (j, series) in sensitivity_series.iter_mut().enumerate().take(p) {
        let block = n + j * n;
        series.push(y[block..block + n].to_vec());
    }
}

/// One classical fourth-order Runge–Kutta step of the augmented system.
///
/// The stage combinations are evaluated in a fixed arithmetic order so the step
/// is bit-reproducible for identical inputs.
fn rk4_step(system: &AugmentedSystem, y: &[f64], dt: f64) -> Result<Vec<f64>, SensitivityError> {
    let half = dt / 2.0;

    let k1 = system.rhs(y)?;
    let k2 = system.rhs(&axpy(y, half, &k1))?;
    let k3 = system.rhs(&axpy(y, half, &k2))?;
    let k4 = system.rhs(&axpy(y, dt, &k3))?;

    let sixth = dt / 6.0;
    let mut next = Vec::with_capacity(y.len());
    for index in 0..y.len() {
        let increment = k1[index] + 2.0 * k2[index] + 2.0 * k3[index] + k4[index];
        next.push(y[index] + sixth * increment);
    }
    Ok(next)
}

/// Returns `y + scale · direction`, element-wise, in a fixed order.
fn axpy(y: &[f64], scale: f64, direction: &[f64]) -> Vec<f64> {
    y.iter().zip(direction).map(|(base, delta)| base + scale * delta).collect()
}
