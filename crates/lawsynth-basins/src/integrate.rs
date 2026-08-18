//! Local, deterministic fixed-step RK4 forward flow.
//!
//! The flow evaluates the ordered field expressions with `lawsynth-expr`; the
//! integrator itself is std-only and performs its floating-point operations in a
//! fixed order, so identical inputs yield bit-identical trajectories.

use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, EvaluationError, Expr, evaluate};

/// An autonomous vector field `ẋ = f(x)` ready for forward integration.
///
/// `fields[i]` is the right-hand side for `states[i]`, so a state vector aligned
/// to `states` evaluates to a derivative vector in the same order.
pub(crate) struct Flow<'a> {
    states: &'a [Identifier],
    fields: Vec<&'a Expr>,
}

impl<'a> Flow<'a> {
    /// Wraps the field expressions, already ordered to match `states`.
    pub(crate) fn new(states: &'a [Identifier], fields: Vec<&'a Expr>) -> Self {
        Self { states, fields }
    }

    /// Evaluates `f(x)` at `state`.
    fn derivative(&self, state: &[f64]) -> Result<Vec<f64>, EvaluationError> {
        let environment: Environment =
            self.states.iter().cloned().zip(state.iter().copied()).collect();
        self.fields.iter().map(|field| evaluate(field, &environment)).collect()
    }

    /// Advances `state` by one classical RK4 step of size `dt`.
    ///
    /// Returns an evaluation error if the field is undefined anywhere along the
    /// step (for example a `log` of a non-positive argument); the caller treats
    /// such a trajectory as having left the valid region.
    pub(crate) fn step(&self, state: &[f64], dt: f64) -> Result<Vec<f64>, EvaluationError> {
        let k1 = self.derivative(state)?;
        let k2 = self.derivative(&axpy(state, dt / 2.0, &k1))?;
        let k3 = self.derivative(&axpy(state, dt / 2.0, &k2))?;
        let k4 = self.derivative(&axpy(state, dt, &k3))?;

        let next = state
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                value + (dt / 6.0) * (k1[index] + 2.0 * k2[index] + 2.0 * k3[index] + k4[index])
            })
            .collect();
        Ok(next)
    }
}

/// Returns `base + scale * delta`, componentwise (a fresh vector; no mutation).
fn axpy(base: &[f64], scale: f64, delta: &[f64]) -> Vec<f64> {
    base.iter().zip(delta).map(|(&b, &d)| b + scale * d).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_expr::{Expr, UnaryOperator};

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn exponential_decay_matches_closed_form() {
        // x' = -x, so x(t) = x0 * e^{-t}. RK4 should track it closely.
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let states = [x];
        let flow = Flow::new(&states, vec![&field]);

        let dt = 0.01;
        let mut state = vec![1.0];
        for _ in 0..100 {
            state = flow.step(&state, dt).unwrap();
        }
        // e^{-1} ≈ 0.3678794; RK4 at dt = 0.01 is accurate to well under 1e-6.
        assert!((state[0] - std::f64::consts::E.recip()).abs() < 1e-6);
    }

    #[test]
    fn a_fixed_point_stays_put_exactly() {
        // x' = -x has a fixed point at 0; starting there must not drift.
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let states = [x];
        let flow = Flow::new(&states, vec![&field]);

        let mut state = vec![0.0];
        for _ in 0..1000 {
            state = flow.step(&state, 0.05).unwrap();
        }
        assert_eq!(state, vec![0.0]);
    }

    #[test]
    fn step_is_deterministic() {
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let states = [x];
        let flow = Flow::new(&states, vec![&field]);
        let a = flow.step(&[0.37], 0.01).unwrap();
        let b = flow.step(&[0.37], 0.01).unwrap();
        assert_eq!(a[0].to_bits(), b[0].to_bits());
    }
}
