use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_features::FeatureLibrary;
use lawsynth_sparse::{RegressionProblem, stlsq_standardized};

use crate::binning::binned_estimates;
use crate::{BinnedEstimate, DiscoveredLaw, LawTerm, SdeConfig, SdeError, SdeModel, StateModel};

/// Discovers the drift `a(x)` and diffusion `b²(x)` of a (diagonal-noise) Itô
/// SDE from one or more noisy sample paths.
///
/// The estimator is the **Kramers–Moyal conditional-moment** method: for each
/// increment `ΔX = X(t+Δt) − X(t)` given `X(t)=x`, the drift is `E[ΔX|x]/Δt`
/// and the diffusion is `E[ΔX²|x]/Δt`. The conditional expectations are formed
/// by **binning** the state space and averaging within each bin. Each trusted
/// bin is then **sparse-regressed** onto a polynomial candidate library to yield
/// a closed-form `drift_law` and `diffusion_law`.
///
/// Both the raw binned table and the fitted laws are returned per state. The
/// whole pipeline is deterministic: identical `(dataset, config)` inputs produce
/// a bit-identical [`SdeModel`].
pub fn discover_sde(dataset: &Dataset, config: &SdeConfig) -> Result<SdeModel, SdeError> {
    config.validate()?;

    let time = dataset.time();
    let rows = time.len();
    if rows < 2 {
        return Err(SdeError::TooFewSamples { rows });
    }
    if config.require_regular_time && !time.is_regular(config.time_regular_tolerance) {
        return Err(SdeError::IrregularTimeAxis);
    }

    if rows % config.trajectories != 0 {
        return Err(SdeError::InvalidConfig(format!(
            "row count {rows} is not divisible by trajectories {}",
            config.trajectories
        )));
    }
    let segment_len = rows / config.trajectories;
    if segment_len < 2 {
        return Err(SdeError::TooFewSamples { rows: segment_len });
    }

    // Reported Δt is the mean within-segment spacing; per-step Δt is used for the
    // moment normalisation (correct even on an irregular grid).
    let timestamps = time.values();
    let mean_dt = mean_within_segment_dt(timestamps, segment_len, config.trajectories);

    let selected = selected_states(dataset, config)?;

    let mut states = Vec::with_capacity(selected.len());
    for id in selected {
        let column = dataset
            .columns()
            .get(&id)
            .ok_or_else(|| SdeError::Internal(format!("missing selected column '{id}'")))?;
        states.push(estimate_state(&id, &column.values, timestamps, segment_len, config)?);
    }

    Ok(SdeModel {
        states,
        dt: mean_dt,
        bin_rule: config.bin_rule,
        increment_count: (segment_len - 1) * config.trajectories,
    })
}

/// The mean within-segment sampling interval, excluding boundary gaps.
fn mean_within_segment_dt(timestamps: &[f64], segment_len: usize, trajectories: usize) -> f64 {
    let mut sum = 0.0;
    let mut count = 0.0;
    for segment in 0..trajectories {
        let base = segment * segment_len;
        for local in 0..segment_len - 1 {
            sum += timestamps[base + local + 1] - timestamps[base + local];
            count += 1.0;
        }
    }
    sum / count
}

/// The states to estimate, always in the dataset's (lexicographic) schema order.
fn selected_states(dataset: &Dataset, config: &SdeConfig) -> Result<Vec<Identifier>, SdeError> {
    if config.state_columns.is_empty() {
        return Ok(dataset.schema().columns);
    }
    for requested in &config.state_columns {
        if !dataset.columns().contains_key(requested) {
            return Err(SdeError::UnknownStateColumn(requested.clone()));
        }
    }
    Ok(dataset
        .schema()
        .columns
        .into_iter()
        .filter(|id| config.state_columns.contains(id))
        .collect())
}

/// Builds the binned table and the two fitted laws for a single state.
fn estimate_state(
    state: &Identifier,
    values: &[f64],
    timestamps: &[f64],
    segment_len: usize,
    config: &SdeConfig,
) -> Result<StateModel, SdeError> {
    let (source, increment, step_dt) =
        within_segment_triples(values, timestamps, segment_len, config.trajectories);

    let bins = binned_estimates(state, &source, &increment, &step_dt, config.bin_rule)?;

    let trusted = bins.iter().filter(|bin| bin.count >= config.min_bin_count).copied();
    let trusted: Vec<BinnedEstimate> = trusted.collect();
    let required = config.library_term_count();
    if trusted.len() < required {
        return Err(SdeError::TooFewPopulatedBins {
            state: state.clone(),
            populated: trusted.len(),
            required,
        });
    }

    let design = design_rows(state, &trusted, config)?;
    let drift_targets = trusted.iter().map(|bin| bin.drift).collect::<Vec<_>>();
    let diffusion_targets = trusted.iter().map(|bin| bin.diffusion).collect::<Vec<_>>();
    // Weighted least squares: the variance of a bin's sample mean scales like
    // `1/count`, so the WLS weight is `count` and the row/target scale is
    // `sqrt(count)`. Uniform weighting is a `sqrt(1)` special case.
    let weights = trusted
        .iter()
        .map(|bin| if config.weight_by_occupancy { (bin.count as f64).sqrt() } else { 1.0 })
        .collect::<Vec<_>>();

    let drift_law = fit_law(&design, drift_targets, &weights, state, config)?;
    let diffusion_law = fit_law(&design, diffusion_targets, &weights, state, config)?;

    Ok(StateModel {
        state: state.clone(),
        trusted_bins: trusted.len(),
        bins,
        drift_law,
        diffusion_law,
    })
}

/// Assembles the aligned `(source X(t), increment ΔX, Δt)` triples, forming an
/// increment only for consecutive rows that lie in the same trajectory segment.
fn within_segment_triples(
    values: &[f64],
    timestamps: &[f64],
    segment_len: usize,
    trajectories: usize,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let capacity = (segment_len - 1) * trajectories;
    let mut source = Vec::with_capacity(capacity);
    let mut increment = Vec::with_capacity(capacity);
    let mut step_dt = Vec::with_capacity(capacity);
    for segment in 0..trajectories {
        let base = segment * segment_len;
        for local in 0..segment_len - 1 {
            let i = base + local;
            source.push(values[i]);
            increment.push(values[i + 1] - values[i]);
            step_dt.push(timestamps[i + 1] - timestamps[i]);
        }
    }
    (source, increment, step_dt)
}

/// Evaluates the polynomial candidate library at each trusted bin centre.
fn design_rows(
    state: &Identifier,
    trusted: &[BinnedEstimate],
    config: &SdeConfig,
) -> Result<Vec<Vec<f64>>, SdeError> {
    let centers = trusted.iter().map(|bin| bin.x_center).collect::<Vec<_>>();
    // A synthetic, strictly-increasing time axis so the centres can be presented
    // as a Dataset to the feature library; it carries no dynamical meaning.
    let axis = TimeAxis::new((0..centers.len()).map(|i| i as f64).collect())
        .map_err(|error| SdeError::Internal(error.to_string()))?;
    let column = NumericColumn::new(state.clone(), centers);
    let mini =
        Dataset::new(axis, [column]).map_err(|error| SdeError::Internal(error.to_string()))?;

    let library = FeatureLibrary::polynomial(
        [state.clone()],
        config.polynomial_degree,
        config.include_constant,
    )
    .map_err(|error| SdeError::Feature(error.to_string()))?;
    let matrix = library.evaluate(&mini).map_err(|error| SdeError::Feature(error.to_string()))?;
    Ok(matrix.rows)
}

/// Sparse-regresses one target (drift or diffusion) onto the design matrix,
/// applying the per-bin `sqrt(count)` weights (weighted least squares).
fn fit_law(
    design: &[Vec<f64>],
    targets: Vec<f64>,
    weights: &[f64],
    state: &Identifier,
    config: &SdeConfig,
) -> Result<DiscoveredLaw, SdeError> {
    let rows = design
        .iter()
        .zip(weights)
        .map(|(row, weight)| row.iter().map(|value| value * weight).collect())
        .collect::<Vec<Vec<f64>>>();
    let weighted_targets =
        targets.iter().zip(weights).map(|(target, weight)| target * weight).collect::<Vec<_>>();
    let problem = RegressionProblem::new(rows, weighted_targets)?;
    let solution = stlsq_standardized(&problem, &config.sparse)?;
    Ok(DiscoveredLaw {
        terms: label_terms(state, &solution.coefficients, config),
        residual_sum_squares: solution.residual_sum_squares,
    })
}

/// Attaches readable labels and monomial powers to the fitted coefficients.
fn label_terms(state: &Identifier, coefficients: &[f64], config: &SdeConfig) -> Vec<LawTerm> {
    let powers: Vec<u32> = if config.include_constant {
        (0..=config.polynomial_degree as u32).collect()
    } else {
        (1..=config.polynomial_degree as u32).collect()
    };
    debug_assert_eq!(powers.len(), coefficients.len());
    powers
        .into_iter()
        .zip(coefficients)
        .map(|(power, &coefficient)| LawTerm {
            label: power_label(state.as_str(), power),
            power,
            coefficient,
        })
        .collect()
}

/// Renders `1`, `x`, or `x^p` for a monomial power.
fn power_label(variable: &str, power: u32) -> String {
    match power {
        0 => "1".to_owned(),
        1 => variable.to_owned(),
        p => format!("{variable}^{p}"),
    }
}
