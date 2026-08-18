//! Orchestration: seed → Newton → dedup → classify.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, Expr, symbols};
use lawsynth_jacobian::{Jacobian, analytic_jacobian};
use lawsynth_koopman::{Matrix, eigen};

use crate::classify::classify;
use crate::config::StabilityConfig;
use crate::error::StabilityError;
use crate::newton::{Outcome, refine};
use crate::report::{FixedPoint, StabilityReport};
use crate::seeds::seed_points;

/// Locates and classifies the fixed points of an autonomous vector field.
///
/// `fields` pairs each state with its right-hand side `dx_i/dt = f_i(x)`;
/// `states` fixes the coordinate ordering. The analysis:
///
/// 1. assembles the analytic Jacobian (reusing `lawsynth-jacobian`),
/// 2. runs deterministic Newton from a fixed lattice of seeds over the search
///    box plus the origin,
/// 3. de-duplicates the converged roots within `dedup_tolerance` and orders them
///    lexicographically,
/// 4. classifies each by the eigenvalues of the Jacobian there (reusing the
///    `lawsynth-koopman` eigensolver).
///
/// The result is deterministic: identical inputs yield a bit-identical report.
///
/// # Errors
///
/// See [`StabilityError`]. Notably the field must be autonomous — every symbol
/// it references must be one of `states` — or [`StabilityError::UnknownSymbol`]
/// is returned.
pub fn analyze_stability(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    config: &StabilityConfig,
) -> Result<StabilityReport, StabilityError> {
    if states.is_empty() {
        return Err(StabilityError::EmptyStateSpace);
    }
    config.validate(states.len())?;

    // Assemble the Jacobian first; this validates the field/state structure
    // (duplicate state, duplicate or missing field, undifferentiable node).
    let jacobian = analytic_jacobian(fields, states)?;

    // The field expressions aligned to `states` (Jacobian assembly guarantees a
    // unique field per state, so the scan always succeeds).
    let ordered_fields = order_fields(fields, states)?;

    // Autonomy check: every symbol used by the field must be a state, otherwise
    // there is no value to evaluate at. Jacobian entries only ever reference a
    // subset of the field's symbols, so checking the fields is sufficient.
    let known: BTreeSet<&Identifier> = states.iter().collect();
    for field in &ordered_fields {
        if let Some(symbol) = symbols(field).into_iter().find(|symbol| !known.contains(symbol)) {
            return Err(StabilityError::UnknownSymbol(symbol));
        }
    }

    let seeds = seed_points(config.search_box(), config.grid_resolution());
    let seeds_total = seeds.len();

    let mut seeds_converged = 0usize;
    let mut roots: Vec<Vec<f64>> = Vec::new();
    for seed in seeds {
        if let Outcome::Converged(root) = refine(&jacobian, &ordered_fields, states, seed, config) {
            seeds_converged += 1;
            if in_search_box(&root, config) {
                roots.push(root);
            }
        }
    }

    let roots = deduplicate(roots, config.dedup_tolerance());

    let mut fixed_points = Vec::with_capacity(roots.len());
    for root in roots {
        let eigenvalues = eigenvalues_at(&jacobian, states, &root)?;
        let classification = classify(&eigenvalues, config.marginal_band());
        fixed_points.push(FixedPoint { coordinates: root, eigenvalues, classification });
    }

    Ok(StabilityReport { states: states.to_vec(), fixed_points, seeds_total, seeds_converged })
}

/// The field expressions in `states` order.
fn order_fields<'a>(
    fields: &'a [(Identifier, Expr)],
    states: &[Identifier],
) -> Result<Vec<&'a Expr>, StabilityError> {
    states
        .iter()
        .map(|state| {
            fields
                .iter()
                .find(|(target, _)| target == state)
                .map(|(_, expression)| expression)
                .ok_or_else(|| {
                    StabilityError::Jacobian(lawsynth_jacobian::JacobianError::MissingField(
                        state.clone(),
                    ))
                })
        })
        .collect()
}

/// Whether `point` lies inside the search box, allowing a `dedup_tolerance`
/// margin so a root that overshot a boundary by a hair is still kept.
fn in_search_box(point: &[f64], config: &StabilityConfig) -> bool {
    let margin = config.dedup_tolerance();
    point.iter().zip(config.search_box()).all(|(&coordinate, &(lower, upper))| {
        coordinate >= lower - margin && coordinate <= upper + margin
    })
}

/// Lexicographic total order on coordinate vectors, using `f64::total_cmp` so
/// the ordering is total and deterministic (no `NaN` ambiguity).
fn lexicographic(a: &[f64], b: &[f64]) -> Ordering {
    for (left, right) in a.iter().zip(b) {
        let ordering = left.total_cmp(right);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

/// Merges roots within `tolerance` (per-coordinate) and returns them in
/// lexicographic order, keeping the first representative of each cluster.
fn deduplicate(mut roots: Vec<Vec<f64>>, tolerance: f64) -> Vec<Vec<f64>> {
    roots.sort_by(|a, b| lexicographic(a, b));
    let mut unique: Vec<Vec<f64>> = Vec::new();
    for root in roots {
        let is_duplicate = unique
            .iter()
            .any(|kept| kept.iter().zip(&root).all(|(&a, &b)| (a - b).abs() <= tolerance));
        if !is_duplicate {
            unique.push(root);
        }
    }
    unique
}

/// Evaluates the Jacobian at `point` and returns its eigenvalues.
fn eigenvalues_at(
    jacobian: &Jacobian,
    states: &[Identifier],
    point: &[f64],
) -> Result<Vec<lawsynth_koopman::Complex>, StabilityError> {
    let environment: Environment = states.iter().cloned().zip(point.iter().copied()).collect();
    let dense = jacobian.evaluate(&environment)?;
    let matrix = Matrix::from_rows(dense).map_err(StabilityError::Eigen)?;
    let decomposition = eigen(&matrix).map_err(StabilityError::Eigen)?;
    Ok(decomposition.values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deduplicate_merges_near_roots_and_sorts() {
        let roots = vec![vec![1.0, 1.0], vec![0.0, 0.0], vec![1.0 + 1e-9, 1.0 - 1e-9]];
        let unique = deduplicate(roots, 1e-6);
        assert_eq!(unique, vec![vec![0.0, 0.0], vec![1.0, 1.0]]);
    }

    #[test]
    fn lexicographic_orders_by_first_differing_coordinate() {
        assert_eq!(lexicographic(&[0.0, 5.0], &[1.0, -5.0]), Ordering::Less);
        assert_eq!(lexicographic(&[1.0, -1.0], &[1.0, 2.0]), Ordering::Less);
    }
}
