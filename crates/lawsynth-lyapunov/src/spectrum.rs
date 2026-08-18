//! The Benettin/QR Lyapunov-spectrum estimator.
//!
//! This is the top-level driver. It integrates the augmented `(x, Q)` system of
//! [`crate::system`] with a shared fixed-step RK4, periodically reorthonormalizes
//! the perturbation frame with the Gram–Schmidt QR of [`crate::linalg`], and
//! averages the accumulated log-expansion factors into the Lyapunov spectrum.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

use crate::config::LyapunovConfig;
use crate::error::LyapunovError;
use crate::linalg::gram_schmidt_qr;
use crate::report::LyapunovReport;
use crate::system::VariationalSystem;

/// Estimates the Lyapunov spectrum of a discovered autonomous field `ẋ = f(x)`.
///
/// The method is Benettin's: evolve the state together with an orthonormal frame
/// `Q` of perturbation vectors under the variational flow `q̇ = J(x)·q`, using the
/// analytic Jacobian `J(x) = ∂f/∂x`; every `k` steps QR-decompose the evolved
/// frame `Q = Q'·R`, keep `Q'` as the new frame, and accumulate `ln R_ii`; after
/// a transient the `i`-th exponent is `λ_i = (Σ ln R_ii) / T` over the averaging
/// window of elapsed time `T`. The exponents are returned sorted descending.
///
/// The state and frame share one RK4 integrator (identical stages), so the
/// spectrum belongs to the reported discrete trajectory. The initial frame is the
/// deterministic identity `Q = I`; no RNG or clock is used, so identical inputs
/// yield a bit-identical report.
///
/// # Arguments
///
/// - `fields`: the discovered vector field, one `(state, f_i)` pair per state.
/// - `states`: the state ordering that indexes the field and Jacobian.
/// - `initial`: the initial state `x(0)`, in `states` order. It should lie in the
///   basin of the attractor whose spectrum is sought; the transient discard lets
///   the trajectory settle onto it.
/// - `config`: the step, step count, reorthonormalization interval, and transient
///   fraction.
///
/// # Errors
///
/// Returns a typed [`LyapunovError`] for an empty state space, a dimension
/// mismatch, a non-finite initial value, a non-autonomous field (a symbol that is
/// not a state), an invalid config, a Jacobian assembly/differentiation failure, a
/// numeric evaluation failure, a blow-up to a non-finite state, or a degenerate
/// perturbation frame.
pub fn lyapunov_spectrum(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    initial: &[f64],
    config: &LyapunovConfig,
) -> Result<LyapunovReport, LyapunovError> {
    config.validate()?;

    if states.is_empty() {
        return Err(LyapunovError::EmptyStateSpace);
    }
    if initial.len() != states.len() {
        return Err(LyapunovError::DimensionMismatch {
            states: states.len(),
            initial: initial.len(),
        });
    }
    for (state, value) in states.iter().zip(initial) {
        if !value.is_finite() {
            return Err(LyapunovError::NonFiniteInput { symbol: state.clone(), value: *value });
        }
    }

    let system = VariationalSystem::assemble(fields, states)?;
    let n = system.dimension();

    // Initial augmented state: x(0) known, frame Q = I (column j is the j-th unit
    // vector). This is deterministic and already orthonormal.
    let mut y = vec![0.0; system.augmented_len()];
    y[..n].copy_from_slice(initial);
    for j in 0..n {
        y[n + j * n + j] = 1.0;
    }

    let dt = config.step();
    let steps = config.steps();
    let interval = config.reorthonormalization_interval();
    let transient_steps = config.transient_steps();

    // Running sum of ln R_ii per frame column, over the post-transient window,
    // and the elapsed time spanned by that window.
    let mut log_sums = vec![0.0; n];
    let mut accumulated_time = 0.0;
    let mut steps_in_interval = 0usize;

    for step in 0..steps {
        y = rk4_step(&system, &y, dt)?;
        ensure_finite(&y)?;
        steps_in_interval += 1;

        // Reorthonormalize every `interval` steps, and always on the final step so
        // the tail interval is not silently dropped.
        let at_reorthonormalization = steps_in_interval == interval || step == steps - 1;
        if at_reorthonormalization {
            let mut columns = extract_columns(&y, n);
            let r_diagonal = gram_schmidt_qr(&mut columns)?;
            write_columns(&mut y, &columns, n);

            // Accumulate only once the interval has ended past the transient. An
            // interval that straddles the boundary is counted whole; keeping the
            // interval short relative to the run bounds this bias.
            if step + 1 > transient_steps {
                for (sum, r) in log_sums.iter_mut().zip(&r_diagonal) {
                    *sum += r.ln();
                }
                accumulated_time += steps_in_interval as f64 * dt;
            }
            steps_in_interval = 0;
        }
    }

    // Because the final step always triggers a reorthonormalization and the
    // transient fraction is strictly below one, at least one interval is always
    // accumulated, so `accumulated_time` is strictly positive here.
    let mut exponents: Vec<f64> = log_sums.iter().map(|sum| sum / accumulated_time).collect();
    // Sort descending with a total float order so ties and NaN-free values are
    // ordered deterministically.
    exponents.sort_by(|a, b| b.total_cmp(a));

    Ok(LyapunovReport::new(exponents, accumulated_time))
}

/// Estimates only the largest Lyapunov exponent of `ẋ = f(x)` — the sign that
/// decides chaos. A convenience wrapper over [`lyapunov_spectrum`].
///
/// # Errors
///
/// Propagates every error of [`lyapunov_spectrum`].
pub fn largest_lyapunov(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    initial: &[f64],
    config: &LyapunovConfig,
) -> Result<f64, LyapunovError> {
    Ok(lyapunov_spectrum(fields, states, initial, config)?.largest())
}

/// One classical fourth-order Runge–Kutta step of the augmented system.
///
/// The stage combinations are evaluated in a fixed arithmetic order so the step
/// is bit-reproducible for identical inputs.
fn rk4_step(system: &VariationalSystem, y: &[f64], dt: f64) -> Result<Vec<f64>, LyapunovError> {
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

/// Rejects a blow-up: any non-finite component of the augmented state.
fn ensure_finite(y: &[f64]) -> Result<(), LyapunovError> {
    if y.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(LyapunovError::NonFiniteState)
    }
}

/// Extracts the `n` frame columns from the augmented vector.
fn extract_columns(y: &[f64], n: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|j| {
            let block = n + j * n;
            y[block..block + n].to_vec()
        })
        .collect()
}

/// Writes the `n` frame columns back into the augmented vector in place.
fn write_columns(y: &mut [f64], columns: &[Vec<f64>], n: usize) {
    for (j, column) in columns.iter().enumerate() {
        let block = n + j * n;
        y[block..block + n].copy_from_slice(column);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_expr::UnaryOperator;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    /// ẋ = -x has the single exact exponent -1.
    #[test]
    fn scalar_decay_exponent_is_minus_one() {
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let config = LyapunovConfig::default().with_steps(4000);
        let report = lyapunov_spectrum(&[(x.clone(), field)], &[x], &[1.0], &config).unwrap();
        assert_eq!(report.dimension(), 1);
        assert!((report.largest() - (-1.0)).abs() < 1e-3, "got {}", report.largest());
    }

    #[test]
    fn dimension_mismatch_is_rejected() {
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let config = LyapunovConfig::default();
        assert_eq!(
            lyapunov_spectrum(&[(x.clone(), field)], &[x], &[1.0, 2.0], &config),
            Err(LyapunovError::DimensionMismatch { states: 1, initial: 2 })
        );
    }

    #[test]
    fn largest_wrapper_agrees_with_spectrum() {
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let config = LyapunovConfig::default().with_steps(2000);
        let largest = largest_lyapunov(
            &[(x.clone(), field.clone())],
            std::slice::from_ref(&x),
            &[1.0],
            &config,
        )
        .unwrap();
        let report = lyapunov_spectrum(&[(x, field)], &[id("x")], &[1.0], &config).unwrap();
        assert_eq!(largest, report.largest());
    }
}
