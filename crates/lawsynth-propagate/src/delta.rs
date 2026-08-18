//! The delta method: first-order (linearised) propagation of `Cov(θ)` through
//! the forward sensitivities into a state covariance.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sensitivity::{SensitivityConfig, SensitivityTrajectory, forward_sensitivities};

use crate::bands::ForecastBands;
use crate::covariance::validate_covariance;
use crate::error::PropagateError;

/// Roundoff slack: a diagonal of `S·Cov·Sᵀ` computed as slightly negative from a
/// legitimately positive-semi-definite covariance is clamped to zero, but a
/// clearly negative value (an indefinite covariance) is reported as an error.
const VARIANCE_FLOOR_TOLERANCE: f64 = 1e-9;

/// Propagate parameter covariance into forecast bands by the delta method.
///
/// The forward sensitivities `S(t) = ∂x(t)/∂θ` are integrated internally (via
/// `lawsynth-sensitivity`) at the supplied `parameter_values`, and the state
/// covariance is the first-order image `Cov(x(t)) ≈ S(t)·Cov(θ)·S(t)ᵀ`. The
/// per-state variance is that product's diagonal, and the band is
/// `x(t) ± z·sqrt(variance)`.
///
/// `cov_theta` is the `p × p` parameter covariance in `parameters` order; `z` is
/// the band multiplier (e.g. `1.959964` for a two-sided 95% Gaussian band — see
/// [`crate::z_for_confidence`]).
///
/// # Errors
///
/// - [`PropagateError::CovarianceDimensionMismatch`] / [`PropagateError::CovarianceNotSquare`]
///   if `cov_theta` is not a `p × p` matrix.
/// - [`PropagateError::NonFiniteValue`] if a covariance entry is not finite;
///   [`PropagateError::NonFiniteMultiplier`] if `z` is not finite.
/// - [`PropagateError::NotPositiveSemiDefinite`] if the propagated variance is
///   meaningfully negative (an indefinite covariance).
/// - [`PropagateError::Sensitivity`] for any failure of the underlying
///   forward-sensitivity integration (unknown symbol, dimension mismatch, …).
#[allow(clippy::too_many_arguments)] // The propagation contract fixes this surface.
pub fn delta_forecast(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    parameters: &[Identifier],
    initial: &[f64],
    parameter_values: &[f64],
    cov_theta: &[Vec<f64>],
    config: &SensitivityConfig,
    z: f64,
) -> Result<ForecastBands, PropagateError> {
    if !z.is_finite() {
        return Err(PropagateError::NonFiniteMultiplier);
    }
    validate_covariance(cov_theta, parameters.len())?;

    let trajectory =
        forward_sensitivities(fields, states, parameters, initial, parameter_values, config)?;

    let dimension = trajectory.dimension();
    let parameter_count = trajectory.parameter_count();
    let sample_count = trajectory.sample_count();

    let mut mean = vec![vec![0.0; sample_count]; dimension];
    let mut variance = vec![vec![0.0; sample_count]; dimension];
    let mut lower = vec![vec![0.0; sample_count]; dimension];
    let mut upper = vec![vec![0.0; sample_count]; dimension];

    for step in 0..sample_count {
        for state in 0..dimension {
            let sensitivity = collect_sensitivity(&trajectory, state, parameter_count, step);
            let quadratic_form = quadratic_form(&sensitivity, cov_theta)?;
            let spread = z * quadratic_form.sqrt();
            let center = trajectory.state_at(step).map(|row| row[state]).unwrap_or(0.0);
            mean[state][step] = center;
            variance[state][step] = quadratic_form;
            lower[state][step] = center - spread;
            upper[state][step] = center + spread;
        }
    }

    Ok(ForecastBands::new(
        trajectory.times().to_vec(),
        trajectory.states().to_vec(),
        mean,
        variance,
        lower,
        upper,
    ))
}

/// The sensitivity row `[∂x_state/∂θ_0, …, ∂x_state/∂θ_{p−1}]` at `step`.
fn collect_sensitivity(
    trajectory: &SensitivityTrajectory,
    state: usize,
    parameter_count: usize,
    step: usize,
) -> Vec<f64> {
    (0..parameter_count)
        .map(|parameter| trajectory.partial(state, parameter, step).unwrap_or(0.0))
        .collect()
}

/// The scalar `sᵀ · Cov · s`, accumulated in a fixed index order.
///
/// A tiny negative result (positive-semi-definite covariance plus roundoff) is
/// clamped to zero; a clearly negative result signals an indefinite covariance.
fn quadratic_form(sensitivity: &[f64], covariance: &[Vec<f64>]) -> Result<f64, PropagateError> {
    let mut total = 0.0;
    for (row, &s_j) in covariance.iter().zip(sensitivity) {
        let inner: f64 = row.iter().zip(sensitivity).map(|(c_jl, &s_l)| c_jl * s_l).sum();
        total += s_j * inner;
    }
    if total < 0.0 {
        if total > -VARIANCE_FLOOR_TOLERANCE {
            return Ok(0.0);
        }
        return Err(PropagateError::NotPositiveSemiDefinite);
    }
    Ok(total)
}
