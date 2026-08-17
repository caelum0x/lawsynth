//! Joint parameter refinement of discovered candidates (§8.5).
//!
//! Sparse discovery fits coefficients to *finite-difference derivatives*, which
//! is least-squares optimal for that surrogate objective but not for the
//! quantity users care about: the forward-simulated trajectory. This pass
//! re-optimizes a candidate's numeric constants against the observed trajectory
//! with the deterministic bounded coordinate search from `lawsynth-opt`.
//!
//! The search starts from the discovered constants and only accepts strict
//! improvements, so the refined trajectory error never exceeds the initial one.

use std::collections::BTreeMap;

use lawsynth_data::Dataset;
use lawsynth_expr::{Environment, Expr, evaluate};
use lawsynth_opt::{CoordinateConfig, ParameterBounds, coordinate_minimize, mean_squared_error};
use lawsynth_world::{ContinuousLaw, World};

use crate::{DiscoveryCandidate, DiscoveryError, ParameterRefinement, RefinementConfig};
use lawsynth_core::Identifier;

/// Finite penalty returned when a proposed parameterization cannot be simulated
/// (a domain error or non-finite state). It is deterministic and large enough to
/// be rejected by the optimizer without introducing a non-finite objective.
const SIMULATION_PENALTY: f64 = 1e30;

/// Refines the numeric constants of one candidate in place.
///
/// Sets [`DiscoveryCandidate::refinement`] to the trajectory-fit summary and, on
/// strict improvement, replaces the candidate's world with one carrying the
/// refined constants. Candidates without any numeric constants are left
/// untouched (with `refinement` remaining `None`).
pub(crate) fn refine_candidate(
    candidate: &mut DiscoveryCandidate,
    dataset: &Dataset,
    states: &[Identifier],
    config: &RefinementConfig,
) -> Result<(), DiscoveryError> {
    // Gather the state laws and the concatenated initial constants; a single
    // pre-order cursor later splits a flat parameter vector back across the laws.
    let mut initial = Vec::new();
    let mut expressions = Vec::with_capacity(states.len());
    for state in states {
        let Some(law) = candidate.world.laws().get(state) else {
            return Ok(());
        };
        collect_constants(&law.expression, &mut initial);
        expressions.push((state.clone(), law.expression.clone()));
    }
    if initial.is_empty() {
        return Ok(());
    }

    let objective = |parameters: &[f64]| -> f64 {
        let laws = substitute_all(&expressions, parameters);
        trajectory_objective(dataset, states, &laws)
    };

    let baseline = objective(&initial);
    if !baseline.is_finite() || baseline >= SIMULATION_PENALTY {
        // The discovered candidate cannot be simulated on this data; leave it be.
        return Ok(());
    }

    // Bounds wide enough that clamping never moves the starting point, which
    // preserves the optimizer's monotone "never worse than initial" guarantee.
    let magnitude = initial.iter().fold(1.0_f64, |acc, value| acc.max(value.abs()));
    let bound = magnitude * 1e3 + 1e3;
    let bounds = ParameterBounds::new(-bound, bound)
        .map_err(|error| DiscoveryError::Refine(error.to_string()))?;
    let coordinate = CoordinateConfig {
        initial_step: config.initial_step,
        minimum_step: config.minimum_step,
        max_iterations: config.max_iterations,
    };
    let result = coordinate_minimize(&initial, bounds, coordinate, objective)
        .map_err(|error| DiscoveryError::Refine(error.to_string()))?;

    if result.objective < baseline {
        let laws = substitute_all(&expressions, &result.parameters);
        candidate.world = World::new(
            candidate.world.variables().values().cloned(),
            candidate.world.parameters().values().cloned(),
            laws.into_iter().map(|(target, expression)| ContinuousLaw::new(target, expression)),
        )
        .map_err(|error| DiscoveryError::World(error.to_string()))?;
    }
    candidate.refinement = Some(ParameterRefinement {
        parameters: result.parameters,
        mse_before: baseline,
        mse_after: result.objective,
        iterations: result.iterations,
    });
    Ok(())
}

/// Rebuilds every state law from a flat parameter vector, consuming constants in
/// the same pre-order they were collected.
fn substitute_all(
    expressions: &[(Identifier, Expr)],
    parameters: &[f64],
) -> Vec<(Identifier, Expr)> {
    let mut cursor = 0usize;
    expressions
        .iter()
        .map(|(target, expression)| {
            (target.clone(), substitute_constants(expression, parameters, &mut cursor))
        })
        .collect()
}

/// Forward-simulates the candidate with an explicit Euler step over the observed
/// time grid and returns the mean-squared trajectory error via
/// [`lawsynth_opt::mean_squared_error`]. Non-state symbols are read from the
/// observations at each step; state symbols use the running simulated value.
fn trajectory_objective(
    dataset: &Dataset,
    states: &[Identifier],
    laws: &[(Identifier, Expr)],
) -> f64 {
    let times = dataset.time().values();
    let sample_count = times.len();
    if sample_count < 2 {
        return SIMULATION_PENALTY;
    }
    let mut current: BTreeMap<Identifier, f64> =
        states.iter().map(|state| (state.clone(), dataset.columns()[state].values[0])).collect();
    let mut predicted = Vec::with_capacity((sample_count - 1) * states.len());
    let mut observed = Vec::with_capacity((sample_count - 1) * states.len());
    for step in 0..sample_count - 1 {
        let dt = times[step + 1] - times[step];
        let mut environment: Environment = dataset
            .columns()
            .iter()
            .map(|(id, column)| (id.clone(), column.values[step]))
            .collect();
        for (id, value) in &current {
            environment.insert(id.clone(), *value);
        }
        let mut next = current.clone();
        for (target, expression) in laws {
            let derivative = match evaluate(expression, &environment) {
                Ok(value) => value,
                Err(_) => return SIMULATION_PENALTY,
            };
            let value = current[target] + dt * derivative;
            if !value.is_finite() {
                return SIMULATION_PENALTY;
            }
            next.insert(target.clone(), value);
        }
        current = next;
        for target in states {
            predicted.push(current[target]);
            observed.push(dataset.columns()[target].values[step + 1]);
        }
    }
    mean_squared_error(&predicted, &observed).unwrap_or(SIMULATION_PENALTY)
}

/// Collects the numeric constants of an expression in deterministic pre-order.
fn collect_constants(expression: &Expr, out: &mut Vec<f64>) {
    match expression {
        Expr::Constant(value) => out.push(*value),
        Expr::Symbol(_) => {}
        Expr::Unary { operand, .. } => collect_constants(operand, out),
        Expr::Binary { left, right, .. } => {
            collect_constants(left, out);
            collect_constants(right, out);
        }
    }
}

/// Rebuilds an expression, replacing each constant with the next value from
/// `values` in the same pre-order [`collect_constants`] uses.
fn substitute_constants(expression: &Expr, values: &[f64], cursor: &mut usize) -> Expr {
    match expression {
        Expr::Constant(_) => {
            let value = values[*cursor];
            *cursor += 1;
            Expr::constant(value)
        }
        Expr::Symbol(identifier) => Expr::symbol(identifier.clone()),
        Expr::Unary { operator, operand } => {
            Expr::unary(*operator, substitute_constants(operand, values, cursor))
        }
        Expr::Binary { operator, left, right } => Expr::binary(
            *operator,
            substitute_constants(left, values, cursor),
            substitute_constants(right, values, cursor),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn collects_and_substitutes_constants_in_matching_order() {
        // 2 * x + 3  ->  constants [2, 3]
        let expression = Expr::sum(
            Expr::product(Expr::constant(2.0), Expr::symbol(id("x"))),
            Expr::constant(3.0),
        );
        let mut constants = Vec::new();
        collect_constants(&expression, &mut constants);
        assert_eq!(constants, vec![2.0, 3.0]);

        let mut cursor = 0;
        let rebuilt = substitute_constants(&expression, &[5.0, 7.0], &mut cursor);
        assert_eq!(cursor, 2);
        let mut rebuilt_constants = Vec::new();
        collect_constants(&rebuilt, &mut rebuilt_constants);
        assert_eq!(rebuilt_constants, vec![5.0, 7.0]);
    }
}
