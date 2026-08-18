use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, Expr, evaluate};

use crate::differentiate::differentiate;
use crate::error::JacobianError;

/// An analytic Jacobian matrix `J[i][j] = ∂f_i/∂x_j` for a vector field.
///
/// The rows and columns follow the exact `states` ordering supplied to
/// [`analytic_jacobian`]; nothing is derived from hash-map iteration order, so
/// the structure is fully deterministic. Each entry is a simplified expression
/// over the same symbols as the input field (states plus any parameters).
#[derive(Clone, Debug, PartialEq)]
pub struct Jacobian {
    states: Vec<Identifier>,
    entries: Vec<Vec<Expr>>,
}

impl Jacobian {
    /// The dimension `n` of the square `n × n` Jacobian.
    pub fn dimension(&self) -> usize {
        self.states.len()
    }

    /// The state ordering that indexes both rows and columns.
    pub fn states(&self) -> &[Identifier] {
        &self.states
    }

    /// The full matrix of derivative expressions, row-major (`[i][j]`).
    pub fn rows(&self) -> &[Vec<Expr>] {
        &self.entries
    }

    /// The symbolic entry `∂f_row/∂x_col`, or `None` if out of bounds.
    pub fn entry(&self, row: usize, col: usize) -> Option<&Expr> {
        self.entries.get(row).and_then(|columns| columns.get(col))
    }

    /// Evaluates every entry at a numeric point, returning a dense `n × n`
    /// matrix. All symbols referenced by any entry (states and parameters) MUST
    /// be present in `environment`; a missing value yields
    /// [`JacobianError::Evaluation`] wrapping an unknown-symbol error.
    pub fn evaluate(&self, environment: &Environment) -> Result<Vec<Vec<f64>>, JacobianError> {
        self.entries
            .iter()
            .map(|row| {
                row.iter()
                    .map(|entry| evaluate(entry, environment).map_err(JacobianError::from))
                    .collect::<Result<Vec<f64>, JacobianError>>()
            })
            .collect()
    }

    /// A stable textual fingerprint of the whole matrix, used to assert
    /// bit-identical structure across runs.
    pub fn to_canonical_string(&self) -> String {
        let mut output = String::new();
        output.push_str("states:");
        for state in &self.states {
            output.push_str(state.as_str());
            output.push(',');
        }
        output.push('\n');
        for (row_index, row) in self.entries.iter().enumerate() {
            for (col_index, entry) in row.iter().enumerate() {
                output.push_str(&format!(
                    "J[{row_index}][{col_index}]={}\n",
                    entry.to_canonical_string()
                ));
            }
        }
        output
    }
}

/// Builds the analytic Jacobian of a discovered vector field.
///
/// `fields` pairs each state's derivative target with its right-hand side,
/// `dx_i/dt = f_i(x, ...)`. `states` fixes the ordering of the `n × n` matrix:
/// row `i` is `f_{states[i]}` and column `j` differentiates with respect to
/// `states[j]`. The field list may be given in any order — it is matched to
/// `states` by identifier — but the output ordering always follows `states`.
///
/// # Errors
///
/// - [`JacobianError::DuplicateState`] if `states` repeats an identifier.
/// - [`JacobianError::DuplicateField`] if two fields share a target.
/// - [`JacobianError::MissingField`] if a state has no field.
/// - [`JacobianError::UnsupportedDerivative`] if an entry cannot be
///   differentiated in closed form.
pub fn analytic_jacobian(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
) -> Result<Jacobian, JacobianError> {
    // Reject a repeated state up front so row/column indexing is unambiguous.
    for (index, state) in states.iter().enumerate() {
        if states[..index].contains(state) {
            return Err(JacobianError::DuplicateState(state.clone()));
        }
    }
    // Reject duplicate field targets; a linear scan keeps ordering deterministic
    // and avoids any hash-map iteration.
    for (index, (target, _)) in fields.iter().enumerate() {
        if fields[..index].iter().any(|(other, _)| other == target) {
            return Err(JacobianError::DuplicateField(target.clone()));
        }
    }

    let mut entries = Vec::with_capacity(states.len());
    for row_state in states {
        let field = fields
            .iter()
            .find(|(target, _)| target == row_state)
            .map(|(_, expression)| expression)
            .ok_or_else(|| JacobianError::MissingField(row_state.clone()))?;

        let mut row = Vec::with_capacity(states.len());
        for column_state in states {
            row.push(differentiate(field, column_state)?.simplify());
        }
        entries.push(row);
    }

    Ok(Jacobian { states: states.to_vec(), entries })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_expr::{BinaryOperator, UnaryOperator};

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn damped_oscillator_has_constant_jacobian() {
        // x' = y ; y' = -x - 0.3 y  ->  J = [[0, 1], [-1, -0.3]]
        let x = id("x");
        let y = id("y");
        let fields = vec![
            (x.clone(), Expr::symbol(y.clone())),
            (
                y.clone(),
                Expr::difference(
                    Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone())),
                    Expr::product(Expr::constant(0.3), Expr::symbol(y.clone())),
                ),
            ),
        ];
        let jacobian = analytic_jacobian(&fields, &[x, y]).unwrap();
        assert_eq!(jacobian.entry(0, 0), Some(&Expr::constant(0.0)));
        assert_eq!(jacobian.entry(0, 1), Some(&Expr::constant(1.0)));
        assert_eq!(jacobian.entry(1, 0), Some(&Expr::constant(-1.0)));
        assert_eq!(jacobian.entry(1, 1), Some(&Expr::constant(-0.3)));
    }

    #[test]
    fn field_order_does_not_change_output() {
        let x = id("x");
        let y = id("y");
        let f_x = (x.clone(), Expr::symbol(y.clone()));
        let f_y = (
            y.clone(),
            Expr::binary(BinaryOperator::Power, Expr::symbol(x.clone()), Expr::constant(2.0)),
        );
        let states = vec![x.clone(), y.clone()];
        let forward = analytic_jacobian(&[f_x.clone(), f_y.clone()], &states).unwrap();
        let reversed = analytic_jacobian(&[f_y, f_x], &states).unwrap();
        assert_eq!(forward.to_canonical_string(), reversed.to_canonical_string());
    }

    #[test]
    fn missing_field_is_rejected() {
        let x = id("x");
        let y = id("y");
        let fields = vec![(x.clone(), Expr::symbol(y.clone()))];
        assert_eq!(
            analytic_jacobian(&fields, &[x, y.clone()]),
            Err(JacobianError::MissingField(y))
        );
    }

    #[test]
    fn duplicate_state_is_rejected() {
        let x = id("x");
        let fields = vec![(x.clone(), Expr::symbol(x.clone()))];
        assert_eq!(
            analytic_jacobian(&fields, &[x.clone(), x.clone()]),
            Err(JacobianError::DuplicateState(x))
        );
    }

    #[test]
    fn duplicate_field_is_rejected() {
        let x = id("x");
        let fields = vec![(x.clone(), Expr::symbol(x.clone())), (x.clone(), Expr::constant(1.0))];
        assert_eq!(
            analytic_jacobian(&fields, std::slice::from_ref(&x)),
            Err(JacobianError::DuplicateField(x))
        );
    }

    #[test]
    fn evaluate_reports_unknown_symbol() {
        let x = id("x");
        let y = id("y");
        // J entry ∂(x*y)/∂x = y, which needs a value for y at evaluation time.
        let fields = vec![
            (x.clone(), Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone()))),
            (y.clone(), Expr::symbol(x.clone())),
        ];
        let jacobian = analytic_jacobian(&fields, &[x.clone(), y.clone()]).unwrap();
        let environment = Environment::from([(x, 1.0)]); // y deliberately missing
        assert!(matches!(jacobian.evaluate(&environment), Err(JacobianError::Evaluation(_))));
    }
}
