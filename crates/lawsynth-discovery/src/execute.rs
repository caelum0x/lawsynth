use std::num::NonZeroUsize;

use lawsynth_core::stable_hash;
use lawsynth_data::Dataset;
use lawsynth_differentiate::differentiate_dataset_with_config;
use lawsynth_expr::{Environment, Expr, parse, print};
use lawsynth_features::FeatureLibrary;
use lawsynth_preprocess::{AppliedTransform, moving_average};
use lawsynth_profile::profile;
use lawsynth_regime::{Segmentation, pelt};
use lawsynth_score::{CandidateMetrics, expression_complexity, selection_stability};
use lawsynth_sparse::{
    RegressionProblem, SparseSolution, TrappingConfig, frols_standardized, sr3_standardized,
    ssr_standardized, stlsq_standardized, trapping_standardized,
};
use lawsynth_stats::{BootstrapConfig, PercentileInterval, bootstrap_indices, percentile_interval};
use lawsynth_symbolic::{Grammar, calibrate_affine, enumerate};
use lawsynth_units::admits_scaled_dimension;
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

use crate::{
    CancellationToken, DimensionalPruningReport, DimensionalUnits, DiscoveryCandidate,
    DiscoveryCheckpoint, DiscoveryConfig, DiscoveryError, DiscoveryResult, SparseMethod,
};

/// Discovers a continuous polynomial World using finite differences and STLSQ.
pub fn discover(
    dataset: &Dataset,
    config: &DiscoveryConfig,
) -> Result<DiscoveryResult, DiscoveryError> {
    discover_cancellable(dataset, config, &CancellationToken::default())
}

/// Runs discovery while checking a cooperative token between deterministic stages.
pub fn discover_cancellable(
    dataset: &Dataset,
    config: &DiscoveryConfig,
    cancellation: &CancellationToken,
) -> Result<DiscoveryResult, DiscoveryError> {
    let mut checkpoint = DiscoveryCheckpoint::new(dataset.fingerprint());
    discover_cancellable_with_checkpoint(dataset, config, cancellation, &mut checkpoint)
}

/// Runs discovery while updating a durable progress checkpoint after each law.
pub fn discover_cancellable_with_checkpoint(
    dataset: &Dataset,
    config: &DiscoveryConfig,
    cancellation: &CancellationToken,
    checkpoint: &mut DiscoveryCheckpoint,
) -> Result<DiscoveryResult, DiscoveryError> {
    // The public single-node path always uses one feature partition, which
    // routes feature evaluation through `FeatureLibrary::evaluate` unchanged.
    let single_node = NonZeroUsize::new(1).expect("one is nonzero");
    run_discovery(dataset, config, cancellation, checkpoint, single_node)
}

/// Shared discovery driver. `feature_partitions` selects how many deterministic
/// column partitions the feature-library evaluation is split across; one keeps
/// the byte-identical single-node path, higher counts use the additive
/// partitioned path in [`crate::distributed`]. Every other stage is identical
/// regardless of partition count, so the [`DiscoveryResult`] is bit-identical.
pub(crate) fn run_discovery(
    dataset: &Dataset,
    config: &DiscoveryConfig,
    cancellation: &CancellationToken,
    checkpoint: &mut DiscoveryCheckpoint,
    feature_partitions: NonZeroUsize,
) -> Result<DiscoveryResult, DiscoveryError> {
    ensure_active(cancellation)?;
    config
        .resource_limits
        .validate_dataset(dataset.time().len(), dataset.columns().len())
        .map_err(|error| DiscoveryError::Resource(error.to_string()))?;
    if !checkpoint.is_compatible_with(dataset.fingerprint()) {
        return Err(DiscoveryError::Checkpoint(
            "checkpoint belongs to a different dataset".to_owned(),
        ));
    }
    let configuration_fingerprint = stable_hash(format!("{config:?}").as_bytes());
    if !checkpoint.ensure_configuration(configuration_fingerprint) {
        return Err(DiscoveryError::Checkpoint(
            "checkpoint belongs to a different discovery configuration".to_owned(),
        ));
    }
    if dataset.time().len() < 3 {
        return Err(DiscoveryError::TooFewSamples);
    }
    if config.state.is_empty() {
        return Err(DiscoveryError::NoStates);
    }
    for state in &config.state {
        if !dataset.columns().contains_key(state) {
            return Err(DiscoveryError::MissingState(state.to_string()));
        }
    }
    let (working_dataset, preprocessing) = if let Some(pipeline) = &config.preprocessing {
        pipeline.apply(dataset).map_err(|error| DiscoveryError::Preprocess(error.to_string()))?
    } else if let Some(radius) = config.smoothing_radius {
        let (dataset, report) = moving_average(dataset, radius)
            .map_err(|error| DiscoveryError::Preprocess(error.to_string()))?;
        (dataset, vec![AppliedTransform::MovingAverage(report)])
    } else {
        (dataset.clone(), Vec::new())
    };
    let input_profile =
        profile(&working_dataset).map_err(|error| DiscoveryError::Profile(error.to_string()))?;
    ensure_active(cancellation)?;
    let derivatives = differentiate_dataset_with_config(&working_dataset, &config.derivative)
        .map_err(|error| DiscoveryError::Differentiate(error.to_string()))?;
    let feature_variables = working_dataset.columns().keys().cloned().collect::<Vec<_>>();
    let mut library =
        FeatureLibrary::polynomial(feature_variables.clone(), config.polynomial_degree, true)
            .map_err(|error| DiscoveryError::Features(error.to_string()))?;
    if config.include_trigonometric {
        library.extend(
            FeatureLibrary::trigonometric(feature_variables.clone())
                .map_err(|error| DiscoveryError::Features(error.to_string()))?,
        );
    }
    if config.include_rational {
        library.extend(
            FeatureLibrary::bounded_rational(feature_variables.clone())
                .map_err(|error| DiscoveryError::Features(error.to_string()))?,
        );
    }
    config
        .resource_limits
        .validate_feature_count(library.terms().len())
        .map_err(|error| DiscoveryError::Resource(error.to_string()))?;
    let matrix =
        crate::distributed::evaluate_library(&library, &working_dataset, feature_partitions)
            .map_err(|error| DiscoveryError::Features(error.to_string()))?;
    ensure_active(cancellation)?;
    // Apply the grammar template prior once to the materialised candidate library.
    // `None` (no prior) admits every column, so the fit below is byte-identical to
    // the pre-template path. The admitted indices are intersected with each state's
    // dimensional admissibility inside the loop.
    let template_selection = match config.template_prior.as_ref() {
        Some(prior) => Some(
            prior
                .admissible(&matrix.terms)
                .map_err(|error| DiscoveryError::Template(error.to_string()))?,
        ),
        None => None,
    };
    let template_columns: Option<&[usize]> =
        template_selection.as_ref().map(|selection| selection.admitted.as_slice());
    let rows = matrix.rows[1..matrix.rows.len() - 1].to_vec();
    let mut laws = Vec::new();
    let mut total_rss = 0.0;
    let mut complexity = 0;
    // Diagnostic tally of dimensional pruning, only surfaced when units are on.
    let mut pruning = DimensionalPruningReport::default();
    for state in &config.state {
        ensure_active(cancellation)?;
        if let Some(saved) = checkpoint.law(state) {
            let expression = parse(&saved.expression).map_err(|error| {
                DiscoveryError::Checkpoint(format!(
                    "invalid cached expression for '{state}': {error}"
                ))
            })?;
            total_rss += saved.residual_sum_squares;
            complexity += expression_complexity(&expression);
            laws.push(ContinuousLaw::new(state.clone(), expression));
            continue;
        }
        let target = derivatives.columns()[state].values[1..derivatives.time().len() - 1].to_vec();
        // Dimensional pruning selects the admissible columns for this state's
        // target derivative dimension. `None` (units disabled, or this state has
        // no declared unit) keeps every column, so the fit is byte-identical to
        // the pre-units path.
        let dimensional =
            admissible_columns(&matrix.terms, config.units.as_ref(), state, &mut pruning);
        // Combine the (global) template admissibility with this state's (per-state)
        // dimensional admissibility. `None` on both keeps every column, unchanged.
        let admissible =
            combine_columns(template_columns, dimensional.as_deref(), matrix.terms.len());
        let (problem_rows, fit_terms) = match &admissible {
            Some(indices) => (
                rows.iter()
                    .map(|row| indices.iter().map(|&index| row[index]).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                indices.iter().map(|&index| &matrix.terms[index]).collect::<Vec<_>>(),
            ),
            None => (rows.clone(), matrix.terms.iter().collect::<Vec<_>>()),
        };
        // The prior (or dimensional prune) admitted no candidate column for this
        // state: emit an honest zero law rather than fabricating structure. The
        // residual is the target's own sum of squares (the zero model's error).
        if fit_terms.is_empty() {
            let residual_sum_squares = target.iter().map(|value| value * value).sum::<f64>();
            total_rss += residual_sum_squares;
            let expression = Expr::constant(0.0);
            complexity += expression_complexity(&expression);
            let checkpoint_expression = print(&expression);
            laws.push(ContinuousLaw::new(state.clone(), expression));
            checkpoint.record_law(state.clone(), checkpoint_expression, residual_sum_squares);
            continue;
        }
        // The trapping solver damps the state's own linear self-feedback term; its
        // column is the fit term named exactly after the state. Other solvers
        // ignore this index, so the default (STLSQ) path is unaffected.
        let diagonal = fit_terms.iter().position(|term| term.name == state.as_str());
        let solution = fit_sparse(
            &RegressionProblem::new(problem_rows, target)
                .map_err(|error| DiscoveryError::Sparse(error.to_string()))?,
            &config.sparse,
            config.sparse_method,
            diagonal,
        )
        .map_err(|error| DiscoveryError::Sparse(error.to_string()))?;
        let residual_sum_squares = solution.residual_sum_squares;
        total_rss += residual_sum_squares;
        let expression = fit_terms
            .iter()
            .zip(solution.coefficients)
            .filter(|(_, coefficient)| coefficient.abs() >= config.sparse.threshold)
            .map(|(term, coefficient)| {
                Expr::product(Expr::constant(coefficient), term.expression.clone())
            })
            .reduce(Expr::sum)
            .unwrap_or_else(|| Expr::constant(0.0))
            .simplify();
        let checkpoint_expression = print(&expression);
        complexity += expression_complexity(&expression);
        laws.push(ContinuousLaw::new(state.clone(), expression));
        checkpoint.record_law(state.clone(), checkpoint_expression, residual_sum_squares);
    }
    let variables = working_dataset
        .columns()
        .keys()
        .map(|id| {
            Variable::new(
                id.clone(),
                if config.state.contains(id) { VariableRole::State } else { VariableRole::Control },
            )
        })
        .collect::<Vec<_>>();
    let world = World::new(variables.clone(), [], laws)
        .map_err(|error| DiscoveryError::World(error.to_string()))?;
    let observations = rows.len() * config.state.len();
    let bootstrap = bootstrap_summary(
        &rows,
        &derivatives,
        &config.state,
        &config.sparse,
        config.sparse_method,
        config.bootstrap.as_ref(),
        cancellation,
    )?;
    let mut candidates = vec![DiscoveryCandidate {
        world,
        metrics: CandidateMetrics {
            mean_squared_error: total_rss / observations as f64,
            complexity,
        },
        bootstrap_mse: bootstrap.as_ref().map(|summary| summary.mse_interval),
        stability: bootstrap.as_ref().map(|summary| summary.stability),
        refinement: None,
    }];
    if let Some(symbolic_config) = &config.symbolic {
        candidates.push(symbolic_candidate(
            &working_dataset,
            &derivatives,
            &config.state,
            &variables,
            symbolic_config,
            config.units.as_ref(),
            &mut pruning,
            cancellation,
        )?);
    }
    config
        .resource_limits
        .validate_candidate_count(candidates.len())
        .map_err(|error| DiscoveryError::Resource(error.to_string()))?;
    if let Some(refine_config) = &config.refine {
        for candidate in &mut candidates {
            ensure_active(cancellation)?;
            crate::refine::refine_candidate(
                candidate,
                &working_dataset,
                &config.state,
                refine_config,
            )?;
        }
    }
    let frontier = DiscoveryResult::compute_frontier(&candidates);
    let regimes = discover_regimes(&working_dataset, &config.state, config.regime.as_ref())?;
    let (dependency_hypothesis, dependency_assumptions) =
        crate::causal::discover_dependency_hypothesis(&working_dataset, config.causal.as_ref())?;
    Ok(DiscoveryResult {
        profile: input_profile,
        preprocessing,
        candidates,
        frontier,
        regimes,
        dependency_hypothesis,
        dependency_assumptions,
        dimensional_pruning: config.units.as_ref().map(|_| pruning),
        template_filter: template_selection.map(|selection| selection.report),
    })
}

/// Intersects the global template-admissible columns with a state's dimensional
/// columns, preserving ascending order. Both inputs (when present) are already
/// ascending index slices over the same `len` columns.
///
/// Returns `None` — "keep every column, unchanged" — only when *both* filters are
/// absent, so the default (no prior, no units) path stays byte-identical.
fn combine_columns(
    template: Option<&[usize]>,
    dimensional: Option<&[usize]>,
    len: usize,
) -> Option<Vec<usize>> {
    match (template, dimensional) {
        (None, None) => None,
        (Some(indices), None) | (None, Some(indices)) => Some(indices.to_vec()),
        (Some(left), Some(right)) => {
            // Intersect two ascending index lists via a membership mask.
            let mut keep = vec![false; len];
            for &index in right {
                keep[index] = true;
            }
            Some(left.iter().copied().filter(|&index| keep[index]).collect())
        }
    }
}

/// Selects the feature columns that are dimensionally admissible for one state's
/// target derivative, recording each decision in `report`.
///
/// Returns `None` — meaning "keep every column, unchanged" — when units are
/// disabled or the state variable has no declared unit, so the default discovery
/// path is byte-identical. Otherwise returns the retained column indices in
/// ascending order. A candidate term is admissible when a free multiplicative
/// coefficient could rescale it to the target dimension; only dimensionally
/// impossible terms (e.g. `sin(x)` for a dimensioned `x`) are rejected.
fn admissible_columns(
    terms: &[lawsynth_features::FeatureTerm],
    units: Option<&DimensionalUnits>,
    state: &lawsynth_core::Identifier,
    report: &mut DimensionalPruningReport,
) -> Option<Vec<usize>> {
    let units = units?;
    let target = units.target_dimension(state)?;
    let mut kept = Vec::new();
    for (index, term) in terms.iter().enumerate() {
        let admissible = admits_scaled_dimension(&term.expression, units.dimensions(), target);
        report.record(!admissible);
        if admissible {
            kept.push(index);
        }
    }
    Some(kept)
}

/// Runs the opt-in regime segmentation pass over the primary state's window.
///
/// Returns `None` when regime detection is disabled. The primary state is the
/// first configured state; segmenting a single deterministic series keeps the
/// result reproducible and cheap. Enabled explicitly so the default discovery
/// path pays no cost.
fn discover_regimes(
    dataset: &Dataset,
    states: &[lawsynth_core::Identifier],
    config: Option<&lawsynth_regime::SegmentationConfig>,
) -> Result<Option<Segmentation>, DiscoveryError> {
    let Some(config) = config else {
        return Ok(None);
    };
    let Some(primary) = states.first() else {
        return Ok(None);
    };
    let series = &dataset.columns()[primary].values;
    pelt(series, *config).map(Some).map_err(|error| DiscoveryError::Regime(error.to_string()))
}

#[allow(clippy::too_many_arguments)]
fn symbolic_candidate(
    dataset: &Dataset,
    derivatives: &Dataset,
    states: &[lawsynth_core::Identifier],
    variables: &[Variable],
    config: &lawsynth_symbolic::SymbolicConfig,
    units: Option<&DimensionalUnits>,
    pruning: &mut DimensionalPruningReport,
    cancellation: &CancellationToken,
) -> Result<DiscoveryCandidate, DiscoveryError> {
    let contexts = (1..dataset.time().len() - 1)
        .map(|row| {
            dataset
                .columns()
                .iter()
                .map(|(id, column)| (id.clone(), column.values[row]))
                .collect::<Environment>()
        })
        .collect::<Vec<_>>();
    let expressions = enumerate(&Grammar::scalar(dataset.columns().keys().cloned()), config);
    let mut laws = Vec::with_capacity(states.len());
    let mut total_mse = 0.0;
    let mut complexity = 0;
    for state in states {
        ensure_active(cancellation)?;
        let target = &derivatives.columns()[state].values[1..derivatives.time().len() - 1];
        // Prune dimensionally-inconsistent enumerated candidates for this state's
        // derivative dimension before any calibration. With units off (or no unit
        // for this state) every candidate is retained, unchanged.
        let admissible: Vec<&Expr> = match units.and_then(|units| units.target_dimension(state)) {
            Some(target_dimension) => {
                let units = units.expect("target dimension implies units are present");
                expressions
                    .iter()
                    .filter(|expression| {
                        let keep = admits_scaled_dimension(
                            expression,
                            units.dimensions(),
                            target_dimension,
                        );
                        pruning.record(!keep);
                        keep
                    })
                    .collect()
            }
            None => expressions.iter().collect(),
        };
        let best = admissible
            .into_iter()
            .filter_map(|expression| calibrate_affine(expression, &contexts, target).ok())
            .min_by(|left, right| {
                left.fit.mean_squared_error.total_cmp(&right.fit.mean_squared_error).then_with(
                    || {
                        left.expression
                            .to_canonical_string()
                            .len()
                            .cmp(&right.expression.to_canonical_string().len())
                    },
                )
            })
            .ok_or_else(|| {
                DiscoveryError::Symbolic(format!("no evaluable symbolic expression for '{state}'"))
            })?;
        total_mse += best.fit.mean_squared_error;
        complexity += expression_complexity(&best.expression);
        laws.push(ContinuousLaw::new(state.clone(), best.expression));
    }
    let world = World::new(variables.to_vec(), [], laws)
        .map_err(|error| DiscoveryError::World(error.to_string()))?;
    Ok(DiscoveryCandidate {
        world,
        metrics: CandidateMetrics {
            mean_squared_error: total_mse / states.len() as f64,
            complexity,
        },
        bootstrap_mse: None,
        stability: None,
        refinement: None,
    })
}

/// Bootstrap uncertainty attached to the sparse candidate: a percentile
/// interval over resampled mean-squared error, plus a selection-stability
/// summary derived from `lawsynth_score::selection_stability`.
struct BootstrapSummary {
    mse_interval: PercentileInterval,
    stability: f64,
}

fn bootstrap_summary(
    rows: &[Vec<f64>],
    derivatives: &Dataset,
    states: &[lawsynth_core::Identifier],
    sparse: &lawsynth_sparse::SparseConfig,
    method: SparseMethod,
    config: Option<&BootstrapConfig>,
    cancellation: &CancellationToken,
) -> Result<Option<BootstrapSummary>, DiscoveryError> {
    let Some(config) = config else {
        return Ok(None);
    };
    let samples = bootstrap_indices(rows.len(), config)
        .map_err(|error| DiscoveryError::Sparse(error.to_string()))?;
    let mut scores = Vec::with_capacity(samples.len());
    // Per-state boolean selection masks, one row per bootstrap replicate, used
    // to quantify how consistently each term survives resampling.
    let mut selections: Vec<Vec<Vec<bool>>> = vec![Vec::new(); states.len()];
    for indices in samples {
        ensure_active(cancellation)?;
        let sampled_rows = indices.iter().map(|index| rows[*index].clone()).collect::<Vec<_>>();
        let mut rss = 0.0;
        for (position, state) in states.iter().enumerate() {
            let values = &derivatives.columns()[state].values;
            let target = indices.iter().map(|index| values[index + 1]).collect::<Vec<_>>();
            let solution = fit_sparse(
                &RegressionProblem::new(sampled_rows.clone(), target)
                    .map_err(|error| DiscoveryError::Sparse(error.to_string()))?,
                sparse,
                method,
                None,
            )
            .map_err(|error| DiscoveryError::Sparse(error.to_string()))?;
            rss += solution.residual_sum_squares;
            selections[position].push(
                solution
                    .coefficients
                    .iter()
                    .map(|coefficient| coefficient.abs() >= sparse.threshold)
                    .collect(),
            );
        }
        scores.push(rss / (indices.len() * states.len()) as f64);
    }
    let mse_interval = percentile_interval(&scores, 0.95)
        .map_err(|error| DiscoveryError::Sparse(error.to_string()))?;
    let stability = selection_stability_summary(&selections)?;
    Ok(Some(BootstrapSummary { mse_interval, stability }))
}

/// Averages the mean pairwise Jaccard agreement of the per-state selection
/// masks into a single stability score in `[0, 1]`.
fn selection_stability_summary(selections: &[Vec<Vec<bool>>]) -> Result<f64, DiscoveryError> {
    let mut total = 0.0;
    let mut counted = 0usize;
    for state in selections {
        if state.is_empty() {
            continue;
        }
        let stability =
            selection_stability(state).map_err(|error| DiscoveryError::Score(error.to_string()))?;
        total += stability.mean_pairwise_jaccard;
        counted += 1;
    }
    Ok(if counted == 0 { 1.0 } else { total / counted as f64 })
}

/// One-sided damping strength applied by the trapping solver to a positive linear
/// self-feedback coefficient. Fixed here so the discovery path stays deterministic.
const TRAPPING_STABILITY_WEIGHT: f64 = 1.0;

fn fit_sparse(
    problem: &RegressionProblem,
    config: &lawsynth_sparse::SparseConfig,
    method: SparseMethod,
    diagonal: Option<usize>,
) -> Result<SparseSolution, lawsynth_sparse::SparseError> {
    match method {
        SparseMethod::Stlsq => stlsq_standardized(problem, config),
        SparseMethod::Sr3 => sr3_standardized(problem, config),
        SparseMethod::Frols => frols_standardized(problem, config),
        SparseMethod::Ssr => ssr_standardized(problem, config),
        SparseMethod::Trapping => trapping_standardized(
            problem,
            &TrappingConfig {
                sparse: config.clone(),
                diagonal,
                stability_weight: TRAPPING_STABILITY_WEIGHT,
            },
        ),
    }
}

fn ensure_active(cancellation: &CancellationToken) -> Result<(), DiscoveryError> {
    if cancellation.is_cancelled() { Err(DiscoveryError::Cancelled) } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lawsynth_core::Identifier;
    use lawsynth_data::{NumericColumn, TimeAxis};
    use lawsynth_expr::evaluate;

    use super::*;

    #[test]
    fn discovers_a_linear_growth_world() {
        let id = Identifier::new("x").unwrap();
        let time = (0..101).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
        let values = time.iter().map(|time| (2.0 * time).exp()).collect::<Vec<_>>();
        let data =
            Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(id.clone(), values)])
                .unwrap();
        let mut config = DiscoveryConfig::new([id.clone()]);
        config.bootstrap =
            Some(lawsynth_stats::BootstrapConfig { replicates: 5, block_size: 4, seed: 7 });
        let mut checkpoint = DiscoveryCheckpoint::new(data.fingerprint());
        let result = discover_cancellable_with_checkpoint(
            &data,
            &config,
            &CancellationToken::default(),
            &mut checkpoint,
        )
        .unwrap();
        let expression = &result.candidates[0].world.laws()[&id].expression;
        assert!(expression.to_canonical_string().contains("2.000"));
        assert!(result.candidates[0].bootstrap_mse.is_some());
        assert_eq!(checkpoint.completed_states().collect::<Vec<_>>(), vec![&id]);
        assert!(checkpoint.law(&id).is_some());
        let resumed = discover_cancellable_with_checkpoint(
            &data,
            &config,
            &CancellationToken::default(),
            &mut checkpoint,
        )
        .unwrap();
        assert_eq!(resumed.candidates[0].world, result.candidates[0].world);
    }

    #[test]
    fn supports_sr3_sparse_discovery() {
        let x = Identifier::new("x").unwrap();
        let time = (0..101).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
        let data = Dataset::new(
            TimeAxis::new(time.clone()).unwrap(),
            [NumericColumn::new(x.clone(), time.iter().map(|time| (2.0 * time).exp()).collect())],
        )
        .unwrap();
        let mut config = DiscoveryConfig::new([x.clone()]);
        config.sparse_method = SparseMethod::Sr3;
        config.sparse.threshold = 0.01;
        let result = discover(&data, &config).unwrap();
        let value = evaluate(
            &result.candidates[0].world.laws()[&x].expression,
            &BTreeMap::from([(x, 1.0)]),
        )
        .unwrap();
        assert!((value - 2.0).abs() < 0.02);
    }

    #[test]
    fn honours_a_pre_cancelled_discovery_request() {
        let token = crate::CancellationToken::default();
        token.cancel();
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0]).unwrap(),
            [NumericColumn::new(Identifier::new("x").unwrap(), vec![1.0, 2.0])],
        )
        .unwrap();
        assert!(matches!(
            discover_cancellable(
                &data,
                &DiscoveryConfig::new([Identifier::new("x").unwrap()]),
                &token
            ),
            Err(DiscoveryError::Cancelled)
        ));
    }

    #[test]
    fn discovers_a_trigonometric_control_law() {
        let x = Identifier::new("x").unwrap();
        let phase = Identifier::new("phase").unwrap();
        let time = (0..401).map(|index| index as f64 * 0.01).collect::<Vec<_>>();
        let data = Dataset::new(
            TimeAxis::new(time.clone()).unwrap(),
            [
                NumericColumn::new(x.clone(), time.iter().map(|value| value.sin()).collect()),
                NumericColumn::new(phase.clone(), time),
            ],
        )
        .unwrap();
        let mut config = DiscoveryConfig::new([x.clone()]);
        config.polynomial_degree = 0;
        config.include_trigonometric = true;
        config.sparse.threshold = 0.1;
        let result = discover(&data, &config).unwrap();
        let expression = &result.candidates[0].world.laws()[&x].expression;
        let value = evaluate(expression, &BTreeMap::from([(x, 0.0), (phase, 0.0)])).unwrap();
        assert!((value - 1.0).abs() < 0.02);
    }

    #[test]
    fn retains_preprocessing_provenance_with_discovery_results() {
        let x = Identifier::new("x").unwrap();
        let values = (0..11).map(|value| value as f64).collect::<Vec<_>>();
        let data = Dataset::new(
            TimeAxis::new(values.clone()).unwrap(),
            [NumericColumn::new(x.clone(), values)],
        )
        .unwrap();
        let mut config = DiscoveryConfig::new([x]);
        config.polynomial_degree = 0;
        config.preprocessing = Some(lawsynth_preprocess::PreprocessPipeline::new([
            lawsynth_preprocess::PreprocessStep::MovingAverage { radius: 1 },
        ]));
        let result = discover(&data, &config).unwrap();
        assert!(matches!(
            result.preprocessing.as_slice(),
            [lawsynth_preprocess::AppliedTransform::MovingAverage(_)]
        ));
    }

    #[test]
    fn rejects_feature_expansion_that_exceeds_configured_resource_limits() {
        let x = Identifier::new("x").unwrap();
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
            [NumericColumn::new(x.clone(), vec![1.0, 2.0, 3.0])],
        )
        .unwrap();
        let mut config = DiscoveryConfig::new([x]);
        config.resource_limits.max_features = 1;
        assert!(matches!(
            discover(&data, &config),
            Err(DiscoveryError::Resource(message)) if message.contains("features limit exceeded")
        ));
    }

    #[test]
    fn includes_a_calibrated_symbolic_branch_on_the_pareto_front() {
        let x = Identifier::new("x").unwrap();
        let time = (0..101).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
        let data = Dataset::new(
            TimeAxis::new(time.clone()).unwrap(),
            [NumericColumn::new(x.clone(), time.iter().map(|time| (2.0 * time).exp()).collect())],
        )
        .unwrap();
        let mut config = DiscoveryConfig::new([x.clone()]);
        config.polynomial_degree = 0;
        config.symbolic = Some(lawsynth_symbolic::SymbolicConfig {
            max_depth: 0,
            max_candidates: 8,
            include_products: false,
        });
        let result = discover(&data, &config).unwrap();
        let symbolic = result
            .candidates
            .iter()
            .find(|candidate| {
                candidate.world.laws()[&x].expression.to_canonical_string().contains("symbol:x")
            })
            .expect("symbolic branch should be retained");
        let value =
            evaluate(&symbolic.world.laws()[&x].expression, &BTreeMap::from([(x, 1.0)])).unwrap();
        assert!((value - 2.0).abs() < 0.02);
    }

    #[test]
    fn recovers_recognizable_lorenz_terms() {
        let x = Identifier::new("x").unwrap();
        let y = Identifier::new("y").unwrap();
        let z = Identifier::new("z").unwrap();
        let mut xv = vec![1.0];
        let mut yv = vec![1.0];
        let mut zv = vec![1.0];
        let dt = 0.001;
        for _ in 0..2_000 {
            let (next_x, next_y, next_z) =
                rk4_lorenz(*xv.last().unwrap(), *yv.last().unwrap(), *zv.last().unwrap(), dt);
            xv.push(next_x);
            yv.push(next_y);
            zv.push(next_z);
        }
        for (index, ((x, y), z)) in xv.iter_mut().zip(&mut yv).zip(&mut zv).enumerate() {
            *x += 1e-7 * (index as f64 * 0.13).sin();
            *y += 1e-7 * (index as f64 * 0.19).cos();
            *z += 1e-7 * (index as f64 * 0.29).sin();
        }
        let time = (0..xv.len()).map(|index| index as f64 * dt).collect();
        let data = Dataset::new(
            TimeAxis::new(time).unwrap(),
            [
                NumericColumn::new(x.clone(), xv),
                NumericColumn::new(y.clone(), yv),
                NumericColumn::new(z.clone(), zv),
            ],
        )
        .unwrap();
        let result = discover(
            &data,
            &DiscoveryConfig {
                state: vec![x.clone(), y.clone(), z.clone()],
                polynomial_degree: 2,
                include_trigonometric: false,
                include_rational: false,
                symbolic: None,
                sparse: lawsynth_sparse::SparseConfig { threshold: 0.1, ..Default::default() },
                sparse_method: SparseMethod::Stlsq,
                derivative: Default::default(),
                smoothing_radius: None,
                preprocessing: None,
                bootstrap: None,
                regime: None,
                refine: None,
                causal: None,
                units: None,
                template_prior: None,
                resource_limits: Default::default(),
            },
        )
        .unwrap();
        let environment = BTreeMap::from([(x.clone(), 1.0), (y.clone(), 1.0), (z.clone(), 1.0)]);
        let laws = result.candidates[0].world.laws();
        assert!(evaluate(&laws[&x].expression, &environment).unwrap().abs() < 0.2);
        assert!((evaluate(&laws[&y].expression, &environment).unwrap() - 26.0).abs() < 0.5);
        assert!((evaluate(&laws[&z].expression, &environment).unwrap() + 5.0 / 3.0).abs() < 0.2);
    }

    #[test]
    fn recovers_recognizable_lotka_volterra_terms() {
        let prey = Identifier::new("prey").unwrap();
        let predator = Identifier::new("predator").unwrap();
        let mut prey_values = vec![10.0];
        let mut predator_values = vec![5.0];
        let dt = 0.001;
        for _ in 0..4_000 {
            let (next_prey, next_predator) =
                rk4_lotka(*prey_values.last().unwrap(), *predator_values.last().unwrap(), dt);
            prey_values.push(next_prey);
            predator_values.push(next_predator);
        }
        for (index, (prey, predator)) in
            prey_values.iter_mut().zip(&mut predator_values).enumerate()
        {
            *prey += 1e-6 * (index as f64 * 0.17).sin();
            *predator += 1e-6 * (index as f64 * 0.31).cos();
        }
        let time = (0..prey_values.len()).map(|index| index as f64 * dt).collect();
        let data = Dataset::new(
            TimeAxis::new(time).unwrap(),
            [
                NumericColumn::new(prey.clone(), prey_values),
                NumericColumn::new(predator.clone(), predator_values),
            ],
        )
        .unwrap();
        let result = discover(
            &data,
            &DiscoveryConfig {
                state: vec![prey.clone(), predator.clone()],
                polynomial_degree: 2,
                include_trigonometric: false,
                include_rational: false,
                symbolic: None,
                sparse: lawsynth_sparse::SparseConfig { threshold: 0.1, ..Default::default() },
                sparse_method: SparseMethod::Stlsq,
                derivative: Default::default(),
                smoothing_radius: None,
                preprocessing: None,
                bootstrap: None,
                regime: None,
                refine: None,
                causal: None,
                units: None,
                template_prior: None,
                resource_limits: Default::default(),
            },
        )
        .unwrap();
        let environment = BTreeMap::from([(prey.clone(), 10.0), (predator.clone(), 5.0)]);
        let laws = result.candidates[0].world.laws();
        assert!((evaluate(&laws[&prey].expression, &environment).unwrap() + 35.0).abs() < 0.5);
        assert!((evaluate(&laws[&predator].expression, &environment).unwrap() - 32.5).abs() < 0.5);
    }

    /// A mechanical oscillator `ẍ = -x`: position `x` in metres, velocity `v` in
    /// m/s, sampled from `x(t) = cos t`, `v(t) = -sin t`. Then `dx/dt = v` and
    /// `dv/dt = -x`, with target dimensions `m/s` and `m/s²`.
    fn oscillator_dataset() -> (Dataset, Identifier, Identifier) {
        let x = Identifier::new("x").unwrap();
        let v = Identifier::new("v").unwrap();
        let time = (0..400).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
        let data = Dataset::new(
            TimeAxis::new(time.clone()).unwrap(),
            [
                NumericColumn::new(x.clone(), time.iter().map(|t| t.cos()).collect()),
                NumericColumn::new(v.clone(), time.iter().map(|t| -t.sin()).collect()),
            ],
        )
        .unwrap();
        (data, x, v)
    }

    fn oscillator_units(x: &Identifier, v: &Identifier) -> crate::DimensionalUnits {
        use lawsynth_units::Unit;
        crate::DimensionalUnits::new([
            (x.clone(), Unit::parse("m").unwrap().dimension()),
            (v.clone(), Unit::parse("m/s").unwrap().dimension()),
        ])
    }

    #[test]
    fn dimensional_pruning_rejects_transcendental_terms_on_dimensioned_inputs() {
        let (data, x, v) = oscillator_dataset();
        let mut config = DiscoveryConfig::new([x.clone(), v.clone()]);
        config.polynomial_degree = 2;
        config.include_trigonometric = true;
        config.sparse.threshold = 0.1;
        config.enable_units(oscillator_units(&x, &v));

        let result = discover(&data, &config).unwrap();
        let report = result.dimensional_pruning.expect("units enable the pruning report");
        // 10 library terms (6 polynomial + 4 sin/cos) tested against 2 states.
        assert_eq!(report.considered, 20);
        // The 4 transcendental terms are impossible for both states: sin/cos of a
        // dimensioned quantity has no consistent dimension.
        assert_eq!(report.pruned, 8);
        assert_eq!(report.retained(), 12);

        // No surviving law may contain a sine or cosine of a dimensioned input.
        let laws = result.candidates[0].world.laws();
        for state in [&x, &v] {
            let canonical = laws[state].expression.to_canonical_string();
            assert!(!canonical.contains("Sin"), "unexpected sine survived: {canonical}");
            assert!(!canonical.contains("Cos"), "unexpected cosine survived: {canonical}");
        }
        // Recovery is preserved: dx/dt = v and dv/dt = -x.
        let dx =
            evaluate(&laws[&x].expression, &BTreeMap::from([(x.clone(), 0.0), (v.clone(), 1.0)]))
                .unwrap();
        let dv =
            evaluate(&laws[&v].expression, &BTreeMap::from([(x.clone(), 1.0), (v.clone(), 0.0)]))
                .unwrap();
        assert!((dx - 1.0).abs() < 0.1, "dx/dt should recover v, got {dx}");
        assert!((dv + 1.0).abs() < 0.1, "dv/dt should recover -x, got {dv}");
    }

    #[test]
    fn units_that_prune_nothing_leave_discovery_byte_identical() {
        let (data, x, v) = oscillator_dataset();
        let mut baseline = DiscoveryConfig::new([x.clone(), v.clone()]);
        baseline.polynomial_degree = 2;
        baseline.sparse.threshold = 0.1;

        // Polynomial-only monomials are always dimensionally consistent (a fit
        // coefficient rescales them), so enabling units prunes nothing and must
        // return exactly the same world as the units-off run.
        let mut with_units = baseline.clone();
        with_units.enable_units(oscillator_units(&x, &v));

        let without = discover(&data, &baseline).unwrap();
        let withu = discover(&data, &with_units).unwrap();

        assert_eq!(without.dimensional_pruning, None);
        let report = withu.dimensional_pruning.expect("units enable the report");
        assert_eq!(report.considered, 12); // 6 monomials x 2 states
        assert_eq!(report.pruned, 0);
        assert_eq!(without.candidates[0].world, withu.candidates[0].world);
    }

    fn rk4_lorenz(x: f64, y: f64, z: f64, dt: f64) -> (f64, f64, f64) {
        let f =
            |x: f64, y: f64, z: f64| (10.0 * (y - x), x * (28.0 - z) - y, x * y - (8.0 / 3.0) * z);
        let (k1x, k1y, k1z) = f(x, y, z);
        let (k2x, k2y, k2z) = f(x + dt * k1x / 2.0, y + dt * k1y / 2.0, z + dt * k1z / 2.0);
        let (k3x, k3y, k3z) = f(x + dt * k2x / 2.0, y + dt * k2y / 2.0, z + dt * k2z / 2.0);
        let (k4x, k4y, k4z) = f(x + dt * k3x, y + dt * k3y, z + dt * k3z);
        (
            x + dt * (k1x + 2.0 * k2x + 2.0 * k3x + k4x) / 6.0,
            y + dt * (k1y + 2.0 * k2y + 2.0 * k3y + k4y) / 6.0,
            z + dt * (k1z + 2.0 * k2z + 2.0 * k3z + k4z) / 6.0,
        )
    }

    fn rk4_lotka(prey: f64, predator: f64, dt: f64) -> (f64, f64) {
        let f = |prey: f64, predator: f64| {
            (1.5 * prey - prey * predator, 0.75 * prey * predator - predator)
        };
        let (k1_prey, k1_predator) = f(prey, predator);
        let (k2_prey, k2_predator) =
            f(prey + dt * k1_prey / 2.0, predator + dt * k1_predator / 2.0);
        let (k3_prey, k3_predator) =
            f(prey + dt * k2_prey / 2.0, predator + dt * k2_predator / 2.0);
        let (k4_prey, k4_predator) = f(prey + dt * k3_prey, predator + dt * k3_predator);
        (
            prey + dt * (k1_prey + 2.0 * k2_prey + 2.0 * k3_prey + k4_prey) / 6.0,
            predator
                + dt * (k1_predator + 2.0 * k2_predator + 2.0 * k3_predator + k4_predator) / 6.0,
        )
    }
}
