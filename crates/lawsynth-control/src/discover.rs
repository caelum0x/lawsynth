use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn};
use lawsynth_differentiate::{DerivativeConfig, differentiate_dataset_with_config};
use lawsynth_sparse::{RegressionProblem, SparseConfig, stlsq_standardized};

use crate::{
    ControlConfig, ControlError, ControlSpec, ControlledModel, StateEquation,
    library::{build_augmented_library, evaluate_library},
};

/// Discovers controlled dynamics `ẋ = Θ(x, u) Ξ` from measured data.
///
/// # Pipeline
///
/// 1. Validate the spec against the dataset (every state/control column exists).
/// 2. Build the augmented library `Θ(x, u)` over `[states.., controls..]` and
///    evaluate it into a shared design matrix.
/// 3. Differentiate **only the state columns** to form the targets `ẋ`.
/// 4. For each state, sparsely regress its derivative onto the augmented library.
///
/// # Controls are inputs, never predicted
///
/// Controls enter step 2 as measured library inputs and never step 3. The model
/// therefore contains exactly one equation per state and none for any control.
///
/// # Determinism
///
/// The variable order is fixed by the spec, the library term order is fixed by
/// `lawsynth-features`, the derivative estimator is deterministic, and
/// `stlsq_standardized` is deterministic. Identical `(dataset, spec, config)`
/// inputs therefore produce bit-identical [`ControlledModel`] output.
pub fn discover_controlled(
    dataset: &Dataset,
    spec: &ControlSpec,
    config: &ControlConfig,
) -> Result<ControlledModel, ControlError> {
    spec.validate_against(dataset)?;

    let library = build_augmented_library(spec, &config.features)?;
    let matrix = evaluate_library(&library, dataset)?;
    let library_terms = matrix.terms.iter().map(|term| term.name.clone()).collect::<Vec<String>>();

    let derivatives = state_derivatives(dataset, spec, &config.derivative)?;

    let equations = spec
        .states()
        .iter()
        .map(|state| {
            let target = derivatives.get(state).expect("state derivative was computed");
            regress_state(state.clone(), &matrix.rows, target, &config.sparse)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ControlledModel {
        equations,
        library_terms,
        // Keep the structured library so forward simulation can evaluate each
        // term's expression tree directly instead of re-parsing label strings.
        library,
        states: spec.states().to_vec(),
        controls: spec.controls().to_vec(),
    })
}

/// Differentiates the state columns (and only the state columns) of `dataset`.
///
/// Controls are deliberately excluded from the sub-dataset handed to the
/// derivative estimator, enforcing the "controls are never differentiated"
/// contract structurally rather than by convention.
fn state_derivatives(
    dataset: &Dataset,
    spec: &ControlSpec,
    config: &DerivativeConfig,
) -> Result<BTreeMap<Identifier, Vec<f64>>, ControlError> {
    let columns = dataset.columns();
    let state_columns = spec
        .states()
        .iter()
        .map(|state| columns.get(state).cloned())
        .collect::<Option<Vec<NumericColumn>>>()
        .ok_or_else(|| {
            // validate_against already ran, so this is unreachable in practice.
            ControlError::UnknownIdentifier("state".into())
        })?;
    let state_dataset = Dataset::new(dataset.time().clone(), state_columns)?;
    let derivative_dataset = differentiate_dataset_with_config(&state_dataset, config)?;
    Ok(derivative_dataset
        .columns()
        .iter()
        .map(|(id, column)| (id.clone(), column.values.clone()))
        .collect())
}

/// Solves the sparse regression `Θ(x, u) ξ ≈ ẋ` for one state derivative.
fn regress_state(
    state: Identifier,
    rows: &[Vec<f64>],
    target: &[f64],
    config: &SparseConfig,
) -> Result<StateEquation, ControlError> {
    if rows.len() != target.len() {
        return Err(ControlError::LengthMismatch { targets: target.len(), rows: rows.len() });
    }
    let problem = RegressionProblem::new(rows.to_vec(), target.to_vec())?;
    let solution = stlsq_standardized(&problem, config)?;
    Ok(StateEquation {
        state,
        coefficients: solution.coefficients,
        residual_sum_squares: solution.residual_sum_squares,
    })
}

#[cfg(test)]
mod tests {
    use lawsynth_data::TimeAxis;

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn linear_dataset() -> Dataset {
        // A tiny, exactly-linear controlled system just for wiring/error tests.
        let time = (0..8).map(|i| i as f64 * 0.1).collect::<Vec<_>>();
        let x = time.iter().map(|t| 0.5 * t).collect::<Vec<_>>();
        let y = time.iter().map(|t| 1.0 - 0.2 * t).collect::<Vec<_>>();
        let u = time.iter().map(|t| 0.3 + 0.1 * t).collect::<Vec<_>>();
        Dataset::new(
            TimeAxis::new(time).unwrap(),
            [
                NumericColumn::new(id("x"), x),
                NumericColumn::new(id("y"), y),
                NumericColumn::new(id("u"), u),
            ],
        )
        .unwrap()
    }

    #[test]
    fn rejects_unknown_state_identifier() {
        let dataset = linear_dataset();
        let spec = ControlSpec::new([id("missing")], [id("u")]).unwrap();
        assert_eq!(
            discover_controlled(&dataset, &spec, &ControlConfig::default()),
            Err(ControlError::UnknownIdentifier("missing".into()))
        );
    }

    #[test]
    fn rejects_unknown_control_identifier() {
        let dataset = linear_dataset();
        let spec = ControlSpec::new([id("x")], [id("nope")]).unwrap();
        assert_eq!(
            discover_controlled(&dataset, &spec, &ControlConfig::default()),
            Err(ControlError::UnknownIdentifier("nope".into()))
        );
    }

    #[test]
    fn regress_state_reports_length_mismatch() {
        let rows = vec![vec![1.0, 0.0], vec![1.0, 1.0]];
        let target = vec![0.0, 1.0, 2.0];
        let error = regress_state(id("x"), &rows, &target, &SparseConfig::default()).unwrap_err();
        assert_eq!(error, ControlError::LengthMismatch { targets: 3, rows: 2 });
    }

    #[test]
    fn produces_one_equation_per_state_and_none_for_controls() {
        let dataset = linear_dataset();
        let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
        let model = discover_controlled(&dataset, &spec, &ControlConfig::default()).unwrap();
        assert_eq!(model.equations.len(), 2);
        assert!(model.equation(&id("x")).is_some());
        assert!(model.equation(&id("y")).is_some());
        // No equation is ever produced for a control.
        assert!(model.equation(&id("u")).is_none());
        // Every equation row aligns with the shared library labels.
        for equation in &model.equations {
            assert_eq!(equation.coefficients.len(), model.library_terms.len());
        }
    }
}
