//! Dependency and causal hypothesis discovery (§8.6).
//!
//! This pass produces a **candidate** causal structure, never a proven-causation
//! claim. It combines the deterministic estimators from `lawsynth-causal`:
//!
//! * [`validate_time_order`] gates the series on a strictly increasing clock, so
//!   any proposed direction respects temporal ordering;
//! * [`pearson_independence`] prunes marginally independent pairs before any edge
//!   is considered;
//! * [`granger_test`] provides the predictive direction — an edge `cause -> effect`
//!   is hypothesized only when the Granger F statistic clears the threshold.
//!
//! The resulting [`CausalGraph`] is reported together with the
//! [`CausalAssumption`]s under which it would license a causal reading (declared
//! via an [`AssumptionSet`] and validated against the graph). Absent those
//! assumptions and stronger identification, the graph is a hypothesis only.

use lawsynth_causal::{
    AssumptionSet, CausalAssumption, CausalConfig, CausalGraph, granger_test, pearson_independence,
    validate_time_order,
};
use lawsynth_data::Dataset;

use crate::{CausalHypothesisConfig, DiscoveryError};

/// Runs the opt-in causal hypothesis pass. Returns the candidate graph and the
/// assumption set it is contingent on, or `None` for both when disabled.
#[allow(clippy::type_complexity)]
pub(crate) fn discover_dependency_hypothesis(
    dataset: &Dataset,
    config: Option<&CausalHypothesisConfig>,
) -> Result<(Option<CausalGraph>, Option<Vec<CausalAssumption>>), DiscoveryError> {
    let Some(config) = config else {
        return Ok((None, None));
    };
    // Honesty gate: a causal reading presumes a valid temporal order.
    validate_time_order(dataset.time().values())
        .map_err(|error| DiscoveryError::Graph(error.to_string()))?;

    let causal_config = CausalConfig {
        max_lag: config.max_lag,
        min_samples: config.min_samples,
        singular_tolerance: 1e-12,
    };
    causal_config.validate().map_err(|error| DiscoveryError::Graph(error.to_string()))?;

    let columns = dataset.columns().iter().collect::<Vec<_>>();
    let mut graph = CausalGraph::new(columns.iter().map(|(id, _)| id.to_string()))
        .map_err(|error| DiscoveryError::Graph(error.to_string()))?;

    // Ordered pairs so both directions are tested; the Granger threshold and the
    // acyclicity guard keep the retained set directed and free of cycles.
    for (cause_index, (cause_id, cause)) in columns.iter().enumerate() {
        for (effect_index, (effect_id, effect)) in columns.iter().enumerate() {
            if cause_index == effect_index {
                continue;
            }
            let independence = pearson_independence(&cause.values, &effect.values)
                .map_err(|error| DiscoveryError::Graph(error.to_string()))?;
            if independence.is_near_independent(config.independence_tolerance) {
                continue;
            }
            let granger = match granger_test(&cause.values, &effect.values, causal_config) {
                Ok(result) => result,
                // Singular design or too few samples is not evidence of an edge.
                Err(_) => continue,
            };
            if granger.f_statistic < config.minimum_f_statistic {
                continue;
            }
            match graph.add_edge(cause_id.to_string(), effect_id.to_string()) {
                Ok(()) => {}
                // Preserve a DAG: if adding this edge would close a cycle, the
                // opposite (already accepted) direction is kept as the hypothesis.
                Err(lawsynth_causal::CausalError::Cycle { .. }) => {}
                Err(error) => return Err(DiscoveryError::Graph(error.to_string())),
            }
        }
    }

    // Declare the assumptions a causal reading would rest on and check they are
    // consistent with the proposed structure. This mirrors the crate's framing:
    // the graph encodes declared assumptions, it does not discover them.
    let mut assumptions = AssumptionSet::default();
    assumptions.declare(CausalAssumption::Faithfulness);
    assumptions.declare(CausalAssumption::CausalSufficiency);
    assumptions
        .validate_against(&graph)
        .map_err(|error| DiscoveryError::Graph(error.to_string()))?;

    let declared = assumptions.iter().cloned().collect::<Vec<_>>();
    Ok((Some(graph), Some(declared)))
}
