//! Deterministic multivariate Newton refinement of a single seed.
//!
//! `x_{k+1} = x_k − J(x_k)^{-1} f(x_k)`, using the analytic Jacobian for `J` and
//! the local Gaussian-elimination solve for the linear system. A seed is dropped
//! (reported as non-convergent) whenever a step cannot be taken: a singular
//! Jacobian, a non-finite or runaway iterate, or a numeric evaluation failure
//! (e.g. `log` of a non-positive value along the path). Nothing is fabricated.

use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, Expr, evaluate};
use lawsynth_jacobian::Jacobian;

use crate::config::StabilityConfig;
use crate::linalg::solve_linear;

/// The outcome of Newton from one seed.
pub(crate) enum Outcome {
    /// Converged to a point whose residual is within tolerance.
    Converged(Vec<f64>),
    /// The seed did not produce a usable fixed point.
    Diverged,
}

fn environment(states: &[Identifier], point: &[f64]) -> Environment {
    states.iter().cloned().zip(point.iter().copied()).collect()
}

fn field_values(fields: &[&Expr], environment: &Environment) -> Option<Vec<f64>> {
    fields.iter().map(|field| evaluate(field, environment).ok()).collect()
}

fn inf_norm(values: &[f64]) -> f64 {
    values.iter().fold(0.0_f64, |worst, &value| worst.max(value.abs()))
}

/// Refines `seed` toward a fixed point using Newton's method.
pub(crate) fn refine(
    jacobian: &Jacobian,
    fields: &[&Expr],
    states: &[Identifier],
    seed: Vec<f64>,
    config: &StabilityConfig,
) -> Outcome {
    let mut point = seed;

    for _ in 0..config.max_iterations() {
        let environment = environment(states, &point);

        let residual = match field_values(fields, &environment) {
            Some(values) => values,
            None => return Outcome::Diverged,
        };
        if inf_norm(&residual) <= config.tolerance() {
            return Outcome::Converged(point);
        }

        let matrix = match jacobian.evaluate(&environment) {
            Ok(matrix) => matrix,
            Err(_) => return Outcome::Diverged,
        };
        let step = match solve_linear(&matrix, &residual) {
            Some(step) => step,
            None => return Outcome::Diverged,
        };

        for (coordinate, delta) in point.iter_mut().zip(&step) {
            *coordinate -= delta;
        }
        if point.iter().any(|value| !value.is_finite() || value.abs() > config.divergence_limit()) {
            return Outcome::Diverged;
        }
    }

    // Budget exhausted: accept only if the final iterate is already a root.
    let environment = environment(states, &point);
    match field_values(fields, &environment) {
        Some(residual) if inf_norm(&residual) <= config.tolerance() => Outcome::Converged(point),
        _ => Outcome::Diverged,
    }
}
