//! The augmented state-and-frame vector field of the variational flow.
//!
//! Given the discovered autonomous fields `f_i(x)`, this module assembles the
//! analytic Jacobian `J(x) = ∂f/∂x` and the ordered state fields, then evaluates
//! the augmented right-hand side
//!
//! ```text
//! ẋ   = f(x)
//! q̇_j = J(x) · q_j        (j = 0 … n-1)
//! ```
//!
//! The augmented vector `y` packs the state first, then one `n`-length block per
//! frame column: `y = [x, q_0, …, q_{n-1}]`, length `n·(1 + n)`. Every
//! perturbation column obeys the same linearized dynamics about the shared
//! trajectory `x(t)`; reorthonormalizing the columns periodically (outside this
//! module) is what turns their growth into the Lyapunov spectrum. Nothing here
//! depends on time or on hash-map iteration order, so evaluation is deterministic.

use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, Expr, evaluate, symbols};
use lawsynth_jacobian::{Jacobian, analytic_jacobian};

use crate::error::LyapunovError;

/// The compiled variational system, ready to evaluate at any `(x, Q)` point.
#[derive(Debug)]
pub(crate) struct VariationalSystem {
    states: Vec<Identifier>,
    /// State fields in `states` order: `ordered_fields[i] = f_i`.
    ordered_fields: Vec<Expr>,
    /// The analytic Jacobian `J(x) = ∂f/∂x` (an `n × n` matrix of expressions).
    jacobian: Jacobian,
}

impl VariationalSystem {
    /// The state-space dimension `n`.
    pub(crate) fn dimension(&self) -> usize {
        self.states.len()
    }

    /// The length `n·(1 + n)` of the augmented vector `[x, q_0, …, q_{n-1}]`.
    pub(crate) fn augmented_len(&self) -> usize {
        let n = self.dimension();
        n * (1 + n)
    }

    /// Assembles the variational system from the discovered fields, validating
    /// the state declarations and the autonomy of the field.
    pub(crate) fn assemble(
        fields: &[(Identifier, Expr)],
        states: &[Identifier],
    ) -> Result<Self, LyapunovError> {
        if states.is_empty() {
            return Err(LyapunovError::EmptyStateSpace);
        }

        // The analytic Jacobian ∂f/∂x. This also validates the field/state
        // structure (duplicate state, duplicate field, missing field) and the
        // differentiability of every entry.
        let jacobian = analytic_jacobian(fields, states)?;

        // The state fields in `states` order. `analytic_jacobian` already proved
        // each state has exactly one field, so the lookup cannot miss here.
        let ordered_fields: Vec<Expr> = states
            .iter()
            .map(|state| {
                fields
                    .iter()
                    .find(|(target, _)| target == state)
                    .map(|(_, expression)| expression.clone())
                    .expect("field presence guaranteed by analytic_jacobian")
            })
            .collect();

        // The field must be autonomous: every symbol it references must be one of
        // the states, otherwise there is no value to bind at evaluation time.
        for field in &ordered_fields {
            for symbol in symbols(field) {
                if !states.contains(&symbol) {
                    return Err(LyapunovError::UnknownSymbol(symbol));
                }
            }
        }

        Ok(Self { states: states.to_vec(), ordered_fields, jacobian })
    }

    /// Builds the evaluation environment binding the supplied state components.
    fn environment(&self, state: &[f64]) -> Environment {
        let mut environment = Environment::new();
        for (identifier, value) in self.states.iter().zip(state) {
            environment.insert(identifier.clone(), *value);
        }
        environment
    }

    /// Evaluates the augmented right-hand side at the augmented point `y`.
    ///
    /// `y` is `[x, q_0, …, q_{n-1}]`; the returned vector holds
    /// `[ẋ, q̇_0, …, q̇_{n-1}]` in the same layout. All accumulation happens in a
    /// fixed index order, so the result is bit-reproducible.
    pub(crate) fn rhs(&self, y: &[f64]) -> Result<Vec<f64>, LyapunovError> {
        let n = self.dimension();

        let state = &y[..n];
        let environment = self.environment(state);

        let mut derivative = vec![0.0; self.augmented_len()];

        // The state derivative ẋ = f(x).
        for (i, field) in self.ordered_fields.iter().enumerate() {
            derivative[i] = evaluate(field, &environment)?;
        }

        // J(x) evaluated once at this point; shared by every frame column.
        let jacobian = self.jacobian.evaluate(&environment)?;

        // q̇_j = J(x) · q_j for each of the n frame columns.
        for j in 0..n {
            let block = n + j * n;
            let column = &y[block..block + n];
            for i in 0..n {
                // (J·q_j)_i = Σ_k J[i][k] · q_j[k].
                let mut accumulator = 0.0;
                for k in 0..n {
                    accumulator += jacobian[i][k] * column[k];
                }
                derivative[block + i] = accumulator;
            }
        }

        Ok(derivative)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_expr::UnaryOperator;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    /// ẋ = -x : J = [-1], so q̇ = -q.
    #[test]
    fn scalar_decay_rhs() {
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let system = VariationalSystem::assemble(&[(x.clone(), field)], &[x]).unwrap();
        // y = [x = 2, q = 1]
        let rhs = system.rhs(&[2.0, 1.0]).unwrap();
        assert_eq!(rhs, vec![-2.0, -1.0]);
    }

    #[test]
    fn rejects_non_autonomous_field() {
        let x = id("x");
        let a = id("a"); // a free symbol, not a state
        let field = Expr::product(Expr::symbol(a.clone()), Expr::symbol(x.clone()));
        let error = VariationalSystem::assemble(&[(x.clone(), field)], &[x]).unwrap_err();
        assert_eq!(error, LyapunovError::UnknownSymbol(a));
    }

    #[test]
    fn rejects_empty_state_space() {
        assert_eq!(
            VariationalSystem::assemble(&[], &[]).unwrap_err(),
            LyapunovError::EmptyStateSpace
        );
    }
}
