use lawsynth_sparse::{RegressionProblem, stlsq_standardized};

use crate::derivatives::{spatial_derivative, time_derivative};
use crate::library::{LibraryTerm, build_terms};
use crate::{PdeConfig, PdeError, PdeModel, PdeTerm};

/// Below this the field is treated as time-invariant (no dynamics to discover).
const DEGENERATE_TIME_DERIVATIVE_RMS: f64 = 1e-12;

/// Discovers a 1-D evolution law `u_t = F(u, u_x, u_xx, ...)` from snapshots of a
/// field `u(x, t)` on a regular space–time grid (PDE-FIND style).
///
/// `field[t][x]` is the value of `u` at time index `t` and spatial index `x`
/// (rows are time snapshots, columns are spatial points). `dx` and `dt` are the
/// uniform spatial and temporal steps.
///
/// The method, in order:
/// 1. estimate `u_t` (central time difference) and the spatial derivatives
///    `u_x, u_xx, ...` (central spatial differences) on the grid **interior**,
///    dropping the outermost points where a central stencil is invalid;
/// 2. build the differential-term candidate library (powers of `u` times
///    derivative factors) row-by-row over the flattened interior;
/// 3. sparse-regress the flattened `u_t` onto that library via STLSQ.
///
/// The interior is visited row-major, **time outer, space inner**:
/// `for t in 1..nt-1 { for x in h..nx-h { .. } }`, where `h` is the spatial
/// half-width of the widest stencil. That fixed order, combined with the
/// deterministic sparse solve, makes identical inputs yield a bit-identical
/// [`PdeModel`].
///
/// # Errors
///
/// Returns a [`PdeError`] for a malformed config or step, a non-rectangular or
/// non-finite field, a grid too small for the central stencils, a field with no
/// time evolution, or a failing sparse solve.
pub fn discover_pde(
    field: &[Vec<f64>],
    dx: f64,
    dt: f64,
    config: &PdeConfig,
) -> Result<PdeModel, PdeError> {
    config.validate()?;
    validate_step("dx", dx)?;
    validate_step("dt", dt)?;
    let (nt, nx) = validate_field(field)?;

    let half_width = config.spatial_half_width();
    if nt < 3 {
        return Err(PdeError::TooFewPoints { axis: "time", have: nt, need: 3 });
    }
    let need_columns = 2 * half_width + 1;
    if nx < need_columns {
        return Err(PdeError::TooFewPoints { axis: "space", have: nx, need: need_columns });
    }

    let terms = build_terms(config);
    let (rows, targets) = assemble_interior(field, dx, dt, half_width, &terms);

    // Relative rescaling: dividing the design matrix and target by RMS(u_t)
    // leaves the least-squares coefficients unchanged (both sides scaled equally)
    // but makes the sparse threshold a dimensionless fraction of the dominant
    // balance. RSS is recovered to original units by multiplying back by scale².
    let target_scale = root_mean_square(&targets);
    if !target_scale.is_finite() {
        return Err(PdeError::Internal("time-derivative scale is not finite".to_owned()));
    }
    if target_scale <= DEGENERATE_TIME_DERIVATIVE_RMS {
        return Err(PdeError::DegenerateField);
    }

    let scaled_rows: Vec<Vec<f64>> =
        rows.iter().map(|row| row.iter().map(|value| value / target_scale).collect()).collect();
    let scaled_targets: Vec<f64> = targets.iter().map(|value| value / target_scale).collect();

    let problem = RegressionProblem::new(scaled_rows, scaled_targets)?;
    let solution = stlsq_standardized(&problem, &config.sparse)?;

    if solution.coefficients.len() != terms.len() {
        return Err(PdeError::Internal(format!(
            "sparse solver returned {} coefficients for {} library terms",
            solution.coefficients.len(),
            terms.len()
        )));
    }

    let fitted_terms = terms
        .iter()
        .zip(&solution.coefficients)
        .map(|(term, &coefficient)| PdeTerm {
            label: term.label.clone(),
            u_power: term.u_power,
            derivative_order: term.derivative_order,
            coefficient,
        })
        .collect();

    Ok(PdeModel {
        variable: config.variable.clone(),
        terms: fitted_terms,
        residual_sum_squares: solution.residual_sum_squares * target_scale * target_scale,
        dx,
        dt,
        interior_points: targets.len(),
        max_u_degree: config.max_u_degree,
        max_derivative_order: config.max_derivative_order,
    })
}

/// Builds the flattened design matrix and `u_t` target over the grid interior.
///
/// Rows are emitted time-outer, space-inner. Each row evaluates every library
/// term `uᵖ · D_m` at that interior `(t, x)`; the zeroth derivative factor is the
/// constant `1`.
fn assemble_interior(
    field: &[Vec<f64>],
    dx: f64,
    dt: f64,
    half_width: usize,
    terms: &[LibraryTerm],
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let nt = field.len();
    let nx = field[0].len();
    let capacity = (nt - 2) * (nx - 2 * half_width);
    let mut rows = Vec::with_capacity(capacity);
    let mut targets = Vec::with_capacity(capacity);

    for t in 1..nt - 1 {
        let snapshot = &field[t];
        for x in half_width..nx - half_width {
            let u = snapshot[x];
            let row = terms
                .iter()
                .map(|term| {
                    let derivative_factor = if term.derivative_order == 0 {
                        1.0
                    } else {
                        spatial_derivative(snapshot, x, term.derivative_order, dx)
                    };
                    u.powi(term.u_power as i32) * derivative_factor
                })
                .collect();
            rows.push(row);
            targets.push(time_derivative(field, t, x, dt));
        }
    }
    (rows, targets)
}

/// Root-mean-square of a slice (`0.0` for an empty slice).
fn root_mean_square(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum_squares = values.iter().map(|value| value * value).sum::<f64>();
    (sum_squares / values.len() as f64).sqrt()
}

/// Ensures a grid step is a finite, strictly positive number.
fn validate_step(name: &str, step: f64) -> Result<(), PdeError> {
    if !step.is_finite() || step <= 0.0 {
        return Err(PdeError::InvalidStep(format!(
            "{name} must be finite and positive, got {step}"
        )));
    }
    Ok(())
}

/// Ensures the field is a non-empty, rectangular, all-finite grid and returns
/// its `(rows, columns)` dimensions.
fn validate_field(field: &[Vec<f64>]) -> Result<(usize, usize), PdeError> {
    let Some(first) = field.first() else {
        return Err(PdeError::EmptyField);
    };
    let nx = first.len();
    if nx == 0 {
        return Err(PdeError::EmptyField);
    }
    for (row_index, row) in field.iter().enumerate() {
        if row.len() != nx {
            return Err(PdeError::NonRectangularField {
                row: row_index,
                expected: nx,
                found: row.len(),
            });
        }
        for (col_index, value) in row.iter().enumerate() {
            if !value.is_finite() {
                return Err(PdeError::NonFiniteValue { row: row_index, col: col_index });
            }
        }
    }
    Ok((field.len(), nx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_non_rectangular_field() {
        let field = vec![vec![0.0, 1.0, 2.0], vec![0.0, 1.0], vec![0.0, 1.0, 2.0]];
        let error = discover_pde(&field, 1.0, 1.0, &PdeConfig::default()).unwrap_err();
        assert!(matches!(error, PdeError::NonRectangularField { row: 1, .. }));
    }

    #[test]
    fn rejects_a_non_finite_field() {
        let field = vec![vec![0.0, 1.0, 2.0], vec![0.0, f64::NAN, 2.0], vec![0.0, 1.0, 2.0]];
        let error = discover_pde(&field, 1.0, 1.0, &PdeConfig::default()).unwrap_err();
        assert!(matches!(error, PdeError::NonFiniteValue { row: 1, col: 1 }));
    }

    #[test]
    fn rejects_a_non_positive_step() {
        let field = vec![vec![0.0, 1.0, 2.0]; 4];
        let error = discover_pde(&field, 0.0, 1.0, &PdeConfig::default()).unwrap_err();
        assert!(matches!(error, PdeError::InvalidStep(_)));
    }

    #[test]
    fn rejects_too_few_time_snapshots() {
        let field = vec![vec![0.0, 1.0, 2.0, 3.0, 4.0]; 2];
        let error = discover_pde(&field, 1.0, 1.0, &PdeConfig::default()).unwrap_err();
        assert!(matches!(error, PdeError::TooFewPoints { axis: "time", .. }));
    }

    #[test]
    fn rejects_too_few_spatial_points() {
        // Order-2 library needs half-width 1 → at least 3 columns; give it 2.
        let field = vec![vec![0.0, 1.0]; 5];
        let error = discover_pde(&field, 1.0, 1.0, &PdeConfig::default()).unwrap_err();
        assert!(matches!(error, PdeError::TooFewPoints { axis: "space", .. }));
    }

    #[test]
    fn rejects_a_time_invariant_field() {
        // Same snapshot at every time step → u_t ≡ 0 → no dynamics.
        let field = vec![(0..8).map(|x| (x as f64).sin()).collect::<Vec<_>>(); 5];
        let error = discover_pde(&field, 0.1, 0.1, &PdeConfig::default()).unwrap_err();
        assert!(matches!(error, PdeError::DegenerateField));
    }

    #[test]
    fn rejects_a_bad_derivative_order() {
        let field = vec![vec![0.0, 1.0, 2.0]; 4];
        let config = PdeConfig::default().with_derivative_order(0);
        assert!(matches!(
            discover_pde(&field, 1.0, 1.0, &config).unwrap_err(),
            PdeError::InvalidConfig(_)
        ));
        let config = PdeConfig::default().with_derivative_order(4);
        assert!(matches!(
            discover_pde(&field, 1.0, 1.0, &config).unwrap_err(),
            PdeError::InvalidConfig(_)
        ));
    }
}
