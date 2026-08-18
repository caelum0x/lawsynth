//! Pairwise symmetry detection via first-derivative invariance.
//!
//! If `f` depends on a variable pair `(x, y)` only through a combination, the
//! gradient satisfies a linear identity everywhere (deterministic, from the
//! field's own numerical partials — no learned probe):
//!
//! | Symmetry   | depends only on | invariant (`≈ 0`)      |
//! |------------|-----------------|------------------------|
//! | Difference | `x − y`         | `f_x + f_y`            |
//! | Sum        | `x + y`         | `f_x − f_y`            |
//! | Product    | `x · y`         | `x·f_x − y·f_y`        |
//! | Ratio      | `x / y`         | `x·f_x + y·f_y`        |

use crate::ReduceError;
use crate::config::ReduceConfig;
use crate::grid::{GridField, rms};
use crate::report::{Symmetry, SymmetryKind, confidence_from_residual};

/// Detects every pairwise symmetry that falls below the tolerance, in a
/// deterministic order (by variable pair, then by symmetry kind).
pub(crate) fn detect(
    field: &GridField,
    config: &ReduceConfig,
) -> Result<Vec<Symmetry>, ReduceError> {
    let mut found = Vec::new();
    let n = field.ndim();
    if n < 2 {
        return Ok(found);
    }
    let partials: Vec<GridField> =
        (0..n).map(|axis| field.partial(axis)).collect::<Result<_, _>>()?;

    for i in 0..n {
        for j in (i + 1)..n {
            found.extend(pair_symmetries(field, &partials[i], &partials[j], i, j, config));
        }
    }

    found.sort_by(|a, b| {
        a.variables.cmp(&b.variables).then_with(|| a.kind.order().cmp(&b.kind.order()))
    });
    Ok(found)
}

fn pair_symmetries(
    field: &GridField,
    fx: &GridField,
    fy: &GridField,
    i: usize,
    j: usize,
    config: &ReduceConfig,
) -> Vec<Symmetry> {
    // Evaluate only on cells interior along both axes, where the numerical
    // partials are central (accurate); endpoints use a one-sided rule.
    let cells = field.interior_cells(&[i, j]);
    let fx_v: Vec<f64> = cells.iter().map(|&c| fx.values[c]).collect();
    let fy_v: Vec<f64> = cells.iter().map(|&c| fy.values[c]).collect();
    let xi: Vec<f64> = cells.iter().map(|&c| field.coord_at(c, i)).collect();
    let yj: Vec<f64> = cells.iter().map(|&c| field.coord_at(c, j)).collect();
    let n = cells.len();

    // Additive-combination residuals share the gradient-magnitude scale.
    let grad_scale = (rms(&fx_v) + rms(&fy_v)).max(1e-30);
    let diff: Vec<f64> = (0..n).map(|c| fx_v[c] + fy_v[c]).collect();
    let sum: Vec<f64> = (0..n).map(|c| fx_v[c] - fy_v[c]).collect();

    // Scaling-combination residuals use the weighted-gradient scale.
    let wx: Vec<f64> = (0..n).map(|c| xi[c] * fx_v[c]).collect();
    let wy: Vec<f64> = (0..n).map(|c| yj[c] * fy_v[c]).collect();
    let weighted_scale = (rms(&wx) + rms(&wy)).max(1e-30);
    let product: Vec<f64> = (0..n).map(|c| wx[c] - wy[c]).collect();
    let ratio: Vec<f64> = (0..n).map(|c| wx[c] + wy[c]).collect();

    let candidates = [
        (SymmetryKind::Difference, rms(&diff) / grad_scale),
        (SymmetryKind::Sum, rms(&sum) / grad_scale),
        (SymmetryKind::Product, rms(&product) / weighted_scale),
        (SymmetryKind::Ratio, rms(&ratio) / weighted_scale),
    ];

    let variables = (field.axes[i].name.clone(), field.axes[j].name.clone());
    candidates
        .into_iter()
        .filter(|&(_, residual)| residual <= config.symmetry_tol)
        .map(|(kind, residual)| Symmetry {
            kind,
            variables: variables.clone(),
            residual,
            confidence: confidence_from_residual(residual),
        })
        .collect()
}
