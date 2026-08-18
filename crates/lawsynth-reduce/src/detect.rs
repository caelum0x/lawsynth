//! Public entry point: reconstruct the grid, then screen + verify reductions.

use lawsynth_data::Dataset;

use crate::config::ReduceConfig;
use crate::grid::{self, GridField};
use crate::report::{GridStatus, ReductionReport};
use crate::{ReduceError, separability, symmetry};

/// Detects deterministic separability and symmetry reductions of a scalar target
/// sampled over the other columns of `dataset`.
///
/// The other columns are treated as the input variables `x1..xn`; the target
/// column is `f`. Detection needs those inputs to form a Cartesian grid so that
/// partials of `f` with respect to one variable can be estimated with the others
/// held fixed. When they do not, the report carries
/// [`GridStatus::NotReconstructed`] and no reductions — never a fabricated one.
///
/// Every reported reduction is a **hypothesis**, carried with the residuals that
/// justify it (see `specs/structural-reductions/README.md`).
pub fn detect_reductions(
    dataset: &Dataset,
    config: &ReduceConfig,
) -> Result<ReductionReport, ReduceError> {
    config.validate()?;

    let schema = dataset.schema();
    let column_names: Vec<String> =
        schema.columns.iter().map(|id| id.as_str().to_string()).collect();

    // Choose the target column.
    let target_name = match &config.target {
        Some(name) => {
            if !column_names.iter().any(|c| c == name) {
                return Err(ReduceError::UnknownTarget { target: name.clone() });
            }
            name.clone()
        }
        None => column_names.last().cloned().ok_or(ReduceError::NoInputColumns)?,
    };

    // Input variables are every other column, in sorted schema order.
    let mut variables: Vec<String> = Vec::new();
    let mut inputs: Vec<(String, Vec<f64>)> = Vec::new();
    let mut target_values: Vec<f64> = Vec::new();
    for id in &schema.columns {
        let name = id.as_str().to_string();
        let values = dataset.columns()[id].values.clone();
        if name == target_name {
            target_values = values;
        } else {
            variables.push(name.clone());
            inputs.push((name, values));
        }
    }
    if inputs.is_empty() {
        return Err(ReduceError::NoInputColumns);
    }
    if inputs.len() > config.max_variables {
        return Err(ReduceError::TooManyVariables {
            available: inputs.len(),
            allowed: config.max_variables,
        });
    }

    // Reconstruct the Cartesian grid; if impossible, report honestly.
    let field: GridField = match grid::reconstruct(
        &inputs,
        &target_values,
        config.min_axis_len,
        config.grid_dedup_rel_tol,
    ) {
        Ok(field) => field,
        Err(failure) => {
            return Ok(ReductionReport {
                target: target_name,
                variables,
                grid: GridStatus::NotReconstructed { reason: failure.reason() },
                separabilities: Vec::new(),
                symmetries: Vec::new(),
            });
        }
    };

    let axis_lengths: Vec<usize> = field.axes.iter().map(|a| a.coords.len()).collect();
    let separabilities = separability::detect(&field, config)?;
    let symmetries = symmetry::detect(&field, config)?;

    Ok(ReductionReport {
        target: target_name,
        variables,
        grid: GridStatus::Reconstructed { axis_lengths },
        separabilities,
        symmetries,
    })
}
