use lawsynth_data::Dataset;
use lawsynth_differentiate::{DerivativeConfig, differentiate_dataset_with_config};

use crate::ImplicitError;
use crate::config::ImplicitConfig;
use crate::library::AugmentedLibrary;
use crate::rational::reconstruct;
use crate::result::{ImplicitDiagnostics, ImplicitResult};
use crate::solve::solve_implicit;

/// Discovers a sparse implicit relation `Θ(x, ẋ) ξ ≈ 0` from a dataset and,
/// when the relation is affine and consistent, the explicit rational law
/// `ẋ = P(x) / Q(x)`.
///
/// The derivative of the target state is estimated from the data with the
/// configured deterministic method; the augmented library is built over the
/// states and that derivative; and the alternating-LHS scheme selects the
/// sparsest consistent relation. The run is fully deterministic and offline.
pub fn implicit_discover(
    dataset: &Dataset,
    config: &ImplicitConfig,
) -> Result<ImplicitResult, ImplicitError> {
    config.validate()?;

    let state_names =
        dataset.columns().keys().map(|id| id.as_str().to_string()).collect::<Vec<_>>();
    let target = resolve_target(dataset, config)?;
    let target_position = state_names
        .iter()
        .position(|name| name == &target)
        .ok_or_else(|| ImplicitError::UnknownTarget(target.clone()))?;

    let derivatives =
        differentiate_dataset_with_config(dataset, &DerivativeConfig { method: config.derivative })
            .map_err(|error| ImplicitError::Differentiation(error.to_string()))?;
    let target_id =
        dataset.columns().keys().nth(target_position).expect("target position is within bounds");
    let xdot_full = &derivatives
        .columns()
        .get(target_id)
        .ok_or_else(|| ImplicitError::UnknownTarget(target.clone()))?
        .values;

    let samples = dataset.time().len();
    let (start, end) = trimmed_range(samples, config.trim_boundary)?;

    let state_rows = (start..end)
        .map(|row| dataset.columns().values().map(|column| column.values[row]).collect())
        .collect::<Vec<Vec<f64>>>();
    let xdot = xdot_full[start..end].to_vec();

    let library =
        AugmentedLibrary::build(&state_names, &target, config.degree, config.include_constant)?;
    let matrix = library.evaluate(&state_rows, &xdot)?;
    let library_size = matrix.terms.len();

    let (relation, candidate_scores) = solve_implicit(&matrix, config)?;

    let rational_law = if relation.consistent {
        reconstruct(&relation, &target, &state_rows, config.min_denominator)
    } else {
        None
    };

    let usable_candidates = candidate_scores.iter().filter(|score| score.usable).count();
    let best_relative_residual = relation.relative_residual;
    let diagnostics = ImplicitDiagnostics {
        target,
        samples: state_rows.len(),
        library_size,
        candidates_evaluated: candidate_scores.len(),
        usable_candidates,
        derivative_method: config.derivative,
        best_relative_residual,
        dataset_fingerprint: dataset.fingerprint(),
        candidate_scores,
    };

    Ok(ImplicitResult { relation, rational_law, diagnostics })
}

fn resolve_target(dataset: &Dataset, config: &ImplicitConfig) -> Result<String, ImplicitError> {
    match &config.target {
        Some(id) => {
            let name = id.as_str().to_string();
            if dataset.columns().contains_key(id) {
                Ok(name)
            } else {
                Err(ImplicitError::UnknownTarget(name))
            }
        }
        None => dataset
            .columns()
            .keys()
            .next()
            .map(|id| id.as_str().to_string())
            .ok_or_else(|| ImplicitError::UnknownTarget(String::new())),
    }
}

fn trimmed_range(samples: usize, trim: bool) -> Result<(usize, usize), ImplicitError> {
    let (start, end) = if trim && samples > 2 { (1, samples - 1) } else { (0, samples) };
    if end.saturating_sub(start) < 3 {
        return Err(ImplicitError::InsufficientSamples);
    }
    Ok((start, end))
}
