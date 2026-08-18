//! The controlled model: nonlinear field for the plant, analytic Jacobian for
//! `A = ∂f/∂x`, and symbolic control partials for `B = ∂f/∂u`.

use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, Expr, evaluate};
use lawsynth_jacobian::{Jacobian, analytic_jacobian, differentiate};
use lawsynth_koopman::Matrix;

use crate::error::MpcError;

/// A parsed, order-resolved control model.
///
/// All symbolic work — matching fields to the `states` ordering, building the
/// analytic Jacobian, and differentiating each field with respect to each
/// control — is done once at construction. The per-step hot loop then only
/// *evaluates* these expressions at numeric points, which keeps the controller
/// deterministic and avoids re-differentiating every step.
pub(crate) struct ControlModel {
    states: Vec<Identifier>,
    controls: Vec<Identifier>,
    /// Field right-hand sides `f_i`, ordered to match `states`.
    fields: Vec<Expr>,
    /// Analytic `∂f/∂x`, an `n × n` Jacobian.
    jacobian: Jacobian,
    /// Simplified control partials `∂f_i/∂u_j`, shaped `n × m` (row-major).
    control_partials: Vec<Vec<Expr>>,
}

impl ControlModel {
    /// Builds a model from a discovered field, a state ordering, and a control
    /// ordering.
    ///
    /// `fields` may be given in any order; each is matched to `states` by
    /// identifier. Every state MUST have exactly one field (enforced by
    /// [`analytic_jacobian`]). Each field is symbolically differentiated with
    /// respect to every control symbol to form `B`'s partials.
    pub(crate) fn build(
        fields: &[(Identifier, Expr)],
        states: &[Identifier],
        controls: &[Identifier],
    ) -> Result<Self, MpcError> {
        if states.is_empty() {
            return Err(MpcError::EmptyStates);
        }
        if controls.is_empty() {
            return Err(MpcError::EmptyControls);
        }

        // Order-resolve fields to the state ordering. `analytic_jacobian` already
        // rejects duplicates and missing fields with typed errors; mirror its
        // lookup so the plant field vector uses the identical ordering.
        let jacobian = analytic_jacobian(fields, states)?;

        let mut ordered_fields = Vec::with_capacity(states.len());
        for state in states {
            let field = fields
                .iter()
                .find(|(target, _)| target == state)
                .map(|(_, expression)| expression.clone())
                .ok_or_else(|| MpcError::Linearization(missing_field(state)))?;
            ordered_fields.push(field);
        }

        // Control partials ∂f_i/∂u_j, simplified once. Differentiation may reject
        // a node it cannot handle in closed form; that surfaces as a typed error.
        let mut control_partials = Vec::with_capacity(states.len());
        for field in &ordered_fields {
            let mut row = Vec::with_capacity(controls.len());
            for control in controls {
                row.push(differentiate(field, control)?.simplify());
            }
            control_partials.push(row);
        }

        Ok(Self {
            states: states.to_vec(),
            controls: controls.to_vec(),
            fields: ordered_fields,
            jacobian,
            control_partials,
        })
    }

    /// The state dimension `n`.
    pub(crate) fn state_dim(&self) -> usize {
        self.states.len()
    }

    /// The control dimension `m`.
    pub(crate) fn control_dim(&self) -> usize {
        self.controls.len()
    }

    /// Binds a `(state, control)` pair into an evaluation environment.
    fn environment(&self, state: &[f64], control: &[f64]) -> Environment {
        let mut environment = Environment::new();
        for (symbol, value) in self.states.iter().zip(state) {
            environment.insert(symbol.clone(), *value);
        }
        for (symbol, value) in self.controls.iter().zip(control) {
            environment.insert(symbol.clone(), *value);
        }
        environment
    }

    /// Evaluates the nonlinear field `f(x, u)` at a point, returning `ẋ` in the
    /// state ordering. Used by the RK4 plant integrator.
    pub(crate) fn field(&self, state: &[f64], control: &[f64]) -> Result<Vec<f64>, MpcError> {
        let environment = self.environment(state, control);
        self.fields
            .iter()
            .map(|expression| evaluate(expression, &environment).map_err(MpcError::from))
            .collect()
    }

    /// Evaluates the linearization `A = ∂f/∂x` at `(x, u)`, as a dense `n × n`
    /// matrix.
    pub(crate) fn state_matrix(&self, state: &[f64], control: &[f64]) -> Result<Matrix, MpcError> {
        let environment = self.environment(state, control);
        let rows = self.jacobian.evaluate(&environment)?;
        Matrix::from_rows(rows).map_err(MpcError::from)
    }

    /// Evaluates the control matrix `B = ∂f/∂u` at `(x, u)`, as a dense `n × m`
    /// matrix.
    pub(crate) fn control_matrix(
        &self,
        state: &[f64],
        control: &[f64],
    ) -> Result<Matrix, MpcError> {
        let environment = self.environment(state, control);
        let mut rows = Vec::with_capacity(self.control_partials.len());
        for partial_row in &self.control_partials {
            let mut row = Vec::with_capacity(partial_row.len());
            for entry in partial_row {
                row.push(evaluate(entry, &environment)?);
            }
            rows.push(row);
        }
        Matrix::from_rows(rows).map_err(MpcError::from)
    }
}

/// Reconstructs the `MissingField` Jacobian error for a state (kept local so the
/// module does not need to import the error's internals elsewhere).
fn missing_field(state: &Identifier) -> lawsynth_jacobian::JacobianError {
    lawsynth_jacobian::JacobianError::MissingField(state.clone())
}
