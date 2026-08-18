//! The augmented state-and-sensitivity vector field.
//!
//! Given the discovered fields `f_i(x; θ)`, this module assembles everything the
//! variational integrator needs — the analytic Jacobian `J_x = ∂f/∂x`, the
//! per-parameter partials `f_{θ_j} = ∂f/∂θ_j`, and the state fields in `states`
//! order — and evaluates the augmented right-hand side
//!
//! ```text
//! ẋ   = f(x; θ)
//! Ṡ_j = J_x · S_j + f_{θ_j}      (j = 1 … p)
//! ```
//!
//! The augmented vector `y` packs the state first, then one `n`-length block per
//! parameter: `y = [x, S_1, …, S_p]`, length `n·(1 + p)`. Nothing here depends on
//! time or on hash-map iteration order, so evaluation is fully deterministic.

use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, Expr, evaluate, symbols};
use lawsynth_jacobian::{Jacobian, analytic_jacobian, differentiate};

use crate::error::SensitivityError;

/// The compiled augmented system, ready to evaluate at any `(x, S)` point.
pub(crate) struct AugmentedSystem {
    states: Vec<Identifier>,
    /// Fixed parameter bindings `(θ_j, value)`, held constant during integration.
    parameter_values: Vec<(Identifier, f64)>,
    /// State fields in `states` order: `ordered_fields[i] = f_i`.
    ordered_fields: Vec<Expr>,
    /// The analytic Jacobian `J_x = ∂f/∂x` (an `n × n` matrix of expressions).
    jacobian: Jacobian,
    /// Parameter partials `partials[j][i] = ∂f_i/∂θ_j`, simplified.
    partials: Vec<Vec<Expr>>,
}

impl AugmentedSystem {
    /// The state-space dimension `n`.
    pub(crate) fn dimension(&self) -> usize {
        self.states.len()
    }

    /// The number of parameters `p`.
    pub(crate) fn parameter_count(&self) -> usize {
        self.parameter_values.len()
    }

    /// The length `n·(1 + p)` of the augmented vector.
    pub(crate) fn augmented_len(&self) -> usize {
        self.dimension() * (1 + self.parameter_count())
    }

    /// Assembles the augmented system from the discovered fields, validating the
    /// state/parameter declarations and computing the analytic partials.
    pub(crate) fn assemble(
        fields: &[(Identifier, Expr)],
        states: &[Identifier],
        parameters: &[Identifier],
        parameter_values: &[f64],
    ) -> Result<Self, SensitivityError> {
        if states.is_empty() {
            return Err(SensitivityError::EmptyStateSpace);
        }
        if parameter_values.len() != parameters.len() {
            return Err(SensitivityError::ParameterDimensionMismatch {
                parameters: parameters.len(),
                values: parameter_values.len(),
            });
        }
        // Reject a repeated parameter so its sensitivity block is unambiguous.
        for (index, parameter) in parameters.iter().enumerate() {
            if parameters[..index].contains(parameter) {
                return Err(SensitivityError::DuplicateParameter(parameter.clone()));
            }
        }
        // A symbol cannot be both an integrated state and a fixed parameter.
        for parameter in parameters {
            if states.contains(parameter) {
                return Err(SensitivityError::ParameterIsState(parameter.clone()));
            }
        }
        // Ensure a non-finite parameter value never enters the environment.
        for (parameter, value) in parameters.iter().zip(parameter_values) {
            if !value.is_finite() {
                return Err(SensitivityError::NonFiniteInput {
                    symbol: parameter.clone(),
                    value: *value,
                });
            }
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

        // Every symbol the fields reference must be a state or a parameter; a free
        // symbol has no value to bind at evaluation time.
        let allowed_states = states;
        let allowed_parameters = parameters;
        for field in &ordered_fields {
            for symbol in symbols(field) {
                let known =
                    allowed_states.contains(&symbol) || allowed_parameters.contains(&symbol);
                if !known {
                    return Err(SensitivityError::UnknownSymbol(symbol));
                }
            }
        }

        // Parameter partials ∂f_i/∂θ_j. A parameter that never appears in a field
        // differentiates to the constant zero, which is exactly the honest "this
        // coefficient does not move the forecast" result.
        let mut partials = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            let mut column = Vec::with_capacity(ordered_fields.len());
            for field in &ordered_fields {
                column.push(differentiate(field, parameter)?.simplify());
            }
            partials.push(column);
        }

        let parameter_values =
            parameters.iter().cloned().zip(parameter_values.iter().copied()).collect();

        Ok(Self { states: states.to_vec(), parameter_values, ordered_fields, jacobian, partials })
    }

    /// Builds the evaluation environment for a given state vector, binding the
    /// fixed parameter values and the supplied state components.
    fn environment(&self, state: &[f64]) -> Environment {
        let mut environment = Environment::new();
        for (identifier, value) in &self.parameter_values {
            environment.insert(identifier.clone(), *value);
        }
        for (identifier, value) in self.states.iter().zip(state) {
            environment.insert(identifier.clone(), *value);
        }
        environment
    }

    /// Evaluates the augmented right-hand side at the augmented point `y`.
    ///
    /// `y` is `[x, S_1, …, S_p]`; the returned vector holds `[ẋ, Ṡ_1, …, Ṡ_p]`
    /// in the same layout. All accumulation happens in a fixed index order, so
    /// the result is bit-reproducible.
    pub(crate) fn rhs(&self, y: &[f64]) -> Result<Vec<f64>, SensitivityError> {
        let n = self.dimension();
        let p = self.parameter_count();

        let state = &y[..n];
        let environment = self.environment(state);

        // The state derivative ẋ = f(x; θ).
        let mut derivative = vec![0.0; self.augmented_len()];
        for (i, field) in self.ordered_fields.iter().enumerate() {
            derivative[i] = evaluate(field, &environment)?;
        }

        // J_x evaluated once at this point; shared by every sensitivity block.
        let jacobian = self.jacobian.evaluate(&environment)?;

        // Ṡ_j = J_x · S_j + f_{θ_j} for each parameter block.
        for j in 0..p {
            let block = n + j * n;
            let sensitivity = &y[block..block + n];
            for i in 0..n {
                // (J_x · S_j)_i = Σ_k J[i][k] · S_j[k].
                let mut accumulator = 0.0;
                for k in 0..n {
                    accumulator += jacobian[i][k] * sensitivity[k];
                }
                let forcing = evaluate(&self.partials[j][i], &environment)?;
                derivative[block + i] = accumulator + forcing;
            }
        }

        Ok(derivative)
    }
}
