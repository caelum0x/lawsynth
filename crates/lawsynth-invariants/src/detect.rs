use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, evaluate, symbols};
use lawsynth_koopman::{Matrix, svd};

use crate::basis::build_basis;
use crate::grid::sample_grid;
use crate::lie::lie_derivative;
use crate::{Invariant, InvariantConfig, InvariantError, InvariantReport};

/// Detects conserved quantities of the autonomous field `ẋ = f(x)`.
///
/// `fields` supplies one right-hand side per state (matched by identifier);
/// `states` fixes the coordinate order. The method parametrizes candidate
/// invariants over the library from [`build_basis`], forms the Lie-derivative
/// matrix `M[k][j] = (L_f φ_j)(x^(k))` over the deterministic sample grid, and
/// reports each near-null right-singular vector of `M` as an invariant.
///
/// See the crate documentation and `specs/invariant-detection/README.md` for the
/// full contract and its honest limits.
pub fn detect_invariants(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    config: &InvariantConfig,
) -> Result<InvariantReport, InvariantError> {
    config.validate()?;
    if states.is_empty() {
        return Err(InvariantError::NoStates);
    }
    if fields.is_empty() {
        return Err(InvariantError::EmptyFields);
    }

    let resolved = resolve_fields(fields, states)?;
    let basis = build_basis(states, config)?;

    // Symbolic Lie derivative of each basis function along the flow.
    let lie: Vec<Expr> = basis
        .iter()
        .map(|term| lie_derivative(&term.expression, &resolved))
        .collect::<Result<_, _>>()?;

    // Sample the Lie derivatives onto the deterministic grid to build M.
    let grid = sample_grid(states, config);
    if grid.len() < basis.len() {
        return Err(InvariantError::InsufficientSamples {
            samples: grid.len(),
            basis: basis.len(),
        });
    }
    let mut rows = Vec::with_capacity(grid.len());
    for environment in &grid {
        let mut row = Vec::with_capacity(lie.len());
        for expression in &lie {
            row.push(evaluate(expression, environment)?);
        }
        rows.push(row);
    }
    let matrix = Matrix::from_rows(rows)?;

    // The right-singular vectors with (near-)zero singular values span the
    // space of conserved quantities.
    let decomposition = svd(&matrix)?;
    let sigma_max = decomposition.s.first().copied().unwrap_or(0.0);
    let threshold = config.tolerance * sigma_max;

    let basis_labels: Vec<String> = basis.iter().map(|term| term.label.clone()).collect();
    let mut invariants = Vec::new();
    for (column, &sigma) in decomposition.s.iter().enumerate() {
        if sigma > threshold {
            continue;
        }
        let coefficients: Vec<f64> =
            (0..basis.len()).map(|row| decomposition.v.get(row, column)).collect();
        let coefficients = normalize(coefficients);
        let residual = norm(&matrix.mat_vec(&coefficients)?);
        invariants.push(Invariant { coefficients, residual, singular_value: sigma });
    }
    // Canonical order: most strongly conserved (smallest singular value) first,
    // with a total order over floats for reproducibility.
    invariants.sort_by(|left, right| left.singular_value.total_cmp(&right.singular_value));

    Ok(InvariantReport { basis_labels, invariants })
}

/// Matches each state to exactly one field, rejecting duplicates, gaps, and
/// fields that reference symbols outside the declared states.
fn resolve_fields<'a>(
    fields: &'a [(Identifier, Expr)],
    states: &'a [Identifier],
) -> Result<Vec<(&'a Identifier, &'a Expr)>, InvariantError> {
    let state_set: BTreeSet<&Identifier> = states.iter().collect();
    let mut resolved = Vec::with_capacity(states.len());
    for (index, state) in states.iter().enumerate() {
        if states[..index].contains(state) {
            return Err(InvariantError::DuplicateState(state.clone()));
        }
        let field = fields
            .iter()
            .find(|(identifier, _)| identifier == state)
            .map(|(_, expression)| expression)
            .ok_or_else(|| InvariantError::MissingField(state.clone()))?;
        for symbol in symbols(field) {
            if !state_set.contains(&symbol) {
                return Err(InvariantError::UnknownSymbol(symbol));
            }
        }
        resolved.push((state, field));
    }
    Ok(resolved)
}

/// Normalizes a coefficient vector to unit norm with a fixed sign convention:
/// the largest-magnitude entry (earliest on ties) is made positive.
fn normalize(mut coefficients: Vec<f64>) -> Vec<f64> {
    let norm = norm(&coefficients);
    if norm == 0.0 {
        return coefficients;
    }
    for coefficient in &mut coefficients {
        *coefficient /= norm;
    }
    let mut pivot = 0;
    let mut pivot_magnitude = 0.0;
    for (index, &coefficient) in coefficients.iter().enumerate() {
        if coefficient.abs() > pivot_magnitude {
            pivot_magnitude = coefficient.abs();
            pivot = index;
        }
    }
    if coefficients[pivot] < 0.0 {
        for coefficient in &mut coefficients {
            *coefficient = -*coefficient;
        }
    }
    coefficients
}

/// The Euclidean norm of a vector.
fn norm(values: &[f64]) -> f64 {
    values.iter().map(|value| value * value).sum::<f64>().sqrt()
}
