//! `lawsynth validate` — "can I trust this model?" forecast-skill diagnostics.
//!
//! Splits observations into a training and a held-out window by time, simulates
//! the world forward across the holdout starting from the observed split point,
//! and scores the forecast per state with RMSE, MAE, R2, and a normalized skill
//! score against a persistence (last-observed-value) baseline.

use std::fmt::Write as _;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_report::format_number;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::World;

use crate::read_numeric_dataset;

const DEFAULT_HOLDOUT: f64 = 0.2;

/// Reusable outcome of a holdout validation, shared with `pipeline`.
pub(crate) struct ValidationSummary {
    /// Human-readable trust verdict.
    pub verdict: String,
    /// Full multi-line report (as `lawsynth validate` prints).
    pub report: String,
}

/// Help text for `lawsynth validate`.
pub fn help() -> String {
    "lawsynth validate WORLD.lsworld --data OBS.{csv,tsv,parquet} [--time COLUMN] [--holdout FRACTION]\n\n\
Splits the observations into train/holdout by time, simulates the world across \
the holdout window from the split point, and reports per-state forecast skill: \
RMSE, MAE, R2, and a normalized skill score versus a persistence baseline. \
Prints a clear trust verdict.\n\n\
Defaults: --time time, --holdout 0.2."
        .to_owned()
}

struct ValidateArgs {
    bundle: String,
    data: String,
    time_column: String,
    holdout: f64,
}

/// Per-state forecast-skill metrics over the holdout window.
struct StateScore {
    state: String,
    rmse: f64,
    mae: f64,
    r_squared: Option<f64>,
    skill: Option<f64>,
}

/// Runs the `validate` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let world = read_world(&args.bundle).map_err(|error| error.to_string())?;
    let dataset = read_numeric_dataset(&args.data, &args.time_column)?;
    let summary = validate_dataset(&world, &dataset, args.holdout, &args.bundle, &args.data)?;
    Ok(summary.report)
}

/// Runs a holdout validation of `world` against `dataset`, returning both the
/// verdict and the full report. Shared by `lawsynth validate` and `pipeline`.
pub(crate) fn validate_dataset(
    world: &World,
    dataset: &Dataset,
    holdout: f64,
    bundle_label: &str,
    data_label: &str,
) -> Result<ValidationSummary, String> {
    let times = dataset.time().values();
    let sample_count = times.len();
    if sample_count < 4 {
        return Err("need at least 4 observations to validate".to_owned());
    }
    if !(0.05..=0.9).contains(&holdout) {
        return Err("holdout must be between 0.05 and 0.9".to_owned());
    }

    // Split by time: train = [0, split), holdout = [split, n).
    let split = ((sample_count as f64) * (1.0 - holdout)).floor() as usize;
    let split = split.clamp(1, sample_count - 2);
    let holdout_len = sample_count - split;

    // The state columns must all be present in the observations.
    let state_ids: Vec<Identifier> = world.state_ids().cloned().collect();
    for state in &state_ids {
        if !dataset.columns().contains_key(state) {
            return Err(format!(
                "state '{}' has no matching column in {}",
                state.as_str(),
                data_label
            ));
        }
    }

    // Initial condition = observed state at the split point (forecast origin).
    let mut request = SimulationRequest::default();
    for state in &state_ids {
        let value = dataset.columns()[state].values[split];
        request = request.with_initial(state.clone(), value);
    }
    let start = times[split];
    let end = times[sample_count - 1];
    let step = holdout_step(times, split);
    let config = SimulationConfig::new(start, end, step).map_err(|error| error.to_string())?;
    let trajectory = simulate(world, config, &request).map_err(|error| error.to_string())?;

    // Score each state by interpolating the simulated trajectory onto the
    // observed holdout timestamps.
    let observed_times = &times[split..sample_count];
    let mut scores = Vec::new();
    for state in &state_ids {
        let simulated = &trajectory.values[state];
        let predicted = interpolate_onto(&trajectory.time, simulated, observed_times);
        let observed = &dataset.columns()[state].values[split..sample_count];
        let origin = dataset.columns()[state].values[split];
        scores.push(score_state(state.as_str(), &predicted, observed, origin));
    }

    let mean_r2 = mean(&scores.iter().filter_map(|score| score.r_squared).collect::<Vec<_>>());
    let mean_skill = mean(&scores.iter().filter_map(|score| score.skill).collect::<Vec<_>>());
    let report =
        render_report(bundle_label, data_label, start, split, holdout_len, holdout, &scores);
    Ok(ValidationSummary { verdict: verdict(mean_r2, mean_skill), report })
}

fn parse(arguments: &[String]) -> Result<ValidateArgs, String> {
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle.starts_with('-') {
        return Err(help());
    }
    let mut data = None;
    let mut time_column = "time".to_owned();
    let mut holdout = DEFAULT_HOLDOUT;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--data" => data = Some(value.clone()),
            "--time" => time_column = value.clone(),
            "--holdout" => {
                holdout = value.parse().map_err(|_| format!("invalid holdout '{value}'"))?;
            }
            _ => return Err(help()),
        }
        index += 2;
    }
    Ok(ValidateArgs {
        bundle: bundle.clone(),
        data: data.ok_or("missing required --data OBS.csv")?,
        time_column,
        holdout,
    })
}

/// Chooses an integration step from the local observation spacing.
fn holdout_step(times: &[f64], split: usize) -> f64 {
    let next = (split + 1).min(times.len() - 1);
    let spacing = times[next] - times[split];
    if spacing.is_finite() && spacing > 0.0 { spacing } else { 1.0 }
}

/// Linearly interpolates `(source_times, source_values)` onto `query_times`.
///
/// Both time series are strictly increasing; queries outside the source range
/// clamp to the nearest endpoint.
fn interpolate_onto(source_times: &[f64], source_values: &[f64], query_times: &[f64]) -> Vec<f64> {
    let mut cursor = 0;
    query_times
        .iter()
        .map(|&query| {
            while cursor + 1 < source_times.len() && source_times[cursor + 1] < query {
                cursor += 1;
            }
            if query <= source_times[0] {
                return source_values[0];
            }
            let last = source_times.len() - 1;
            if query >= source_times[last] {
                return source_values[last];
            }
            let left = cursor;
            let right = (cursor + 1).min(last);
            let span = source_times[right] - source_times[left];
            if span <= 0.0 {
                return source_values[left];
            }
            let fraction = (query - source_times[left]) / span;
            source_values[left] + fraction * (source_values[right] - source_values[left])
        })
        .collect()
}

fn score_state(state: &str, predicted: &[f64], observed: &[f64], origin: f64) -> StateScore {
    let count = observed.len().min(predicted.len());
    let mean = observed.iter().take(count).sum::<f64>() / count as f64;

    let mut sum_squared_error = 0.0;
    let mut sum_absolute_error = 0.0;
    let mut total_variance = 0.0;
    let mut baseline_squared_error = 0.0;
    for index in 0..count {
        let residual = predicted[index] - observed[index];
        sum_squared_error += residual * residual;
        sum_absolute_error += residual.abs();
        total_variance += (observed[index] - mean).powi(2);
        // Persistence baseline predicts the last observed (origin) value.
        baseline_squared_error += (origin - observed[index]).powi(2);
    }
    let rmse = (sum_squared_error / count as f64).sqrt();
    let mae = sum_absolute_error / count as f64;
    let r_squared =
        if total_variance > 0.0 { Some(1.0 - sum_squared_error / total_variance) } else { None };
    let skill = if baseline_squared_error > 0.0 {
        Some(1.0 - (sum_squared_error / baseline_squared_error).sqrt())
    } else {
        None
    };
    StateScore { state: state.to_owned(), rmse, mae, r_squared, skill }
}

fn render_report(
    bundle_label: &str,
    data_label: &str,
    start: f64,
    split: usize,
    holdout_len: usize,
    holdout_fraction: f64,
    scores: &[StateScore],
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Validation: {bundle_label} on {data_label}");
    let _ = writeln!(
        out,
        "  split at t={} | train={} rows | holdout={} rows (fraction {:.2})",
        format_number(start),
        split,
        holdout_len,
        holdout_fraction
    );
    out.push('\n');
    let _ = writeln!(
        out,
        "  {:<12} {:>12} {:>12} {:>10} {:>14}",
        "state", "RMSE", "MAE", "R2", "skill_vs_persist"
    );
    for score in scores {
        let _ = writeln!(
            out,
            "  {:<12} {:>12} {:>12} {:>10} {:>14}",
            score.state,
            format!("{:.4e}", score.rmse),
            format!("{:.4e}", score.mae),
            score.r_squared.map(|value| format!("{value:.4}")).unwrap_or_else(|| "n/a".to_owned()),
            score.skill.map(|value| format!("{value:.4}")).unwrap_or_else(|| "n/a".to_owned()),
        );
    }
    out.push('\n');

    let r_squared: Vec<f64> = scores.iter().filter_map(|score| score.r_squared).collect();
    let skills: Vec<f64> = scores.iter().filter_map(|score| score.skill).collect();
    let mean_r2 = mean(&r_squared);
    let mean_skill = mean(&skills);
    if let Some(mean_r2) = mean_r2 {
        let _ = write!(out, "  aggregate  R2={mean_r2:.4}");
        if let Some(mean_skill) = mean_skill {
            let _ = write!(out, "  skill={mean_skill:.4}");
        }
        out.push('\n');
    }

    let _ = writeln!(out, "Verdict: {}", verdict(mean_r2, mean_skill));
    out
}

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() { None } else { Some(values.iter().sum::<f64>() / values.len() as f64) }
}

fn verdict(mean_r2: Option<f64>, mean_skill: Option<f64>) -> String {
    let Some(r2) = mean_r2 else {
        return "INCONCLUSIVE - held-out states are constant, no variance to score".to_owned();
    };
    let beats = mean_skill.map(|skill| skill > 0.0).unwrap_or(true);
    let base = if r2 >= 0.99 && beats {
        "STRONG - the model tracks held-out data closely"
    } else if r2 >= 0.9 && beats {
        "GOOD - the model generalizes to held-out data"
    } else if r2 >= 0.5 {
        "FAIR - partial predictive skill on held-out data"
    } else {
        "WEAK - the model does not reliably predict held-out data"
    };
    if !beats { format!("{base} (does not beat a persistence baseline)") } else { base.to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_onto_observed_grid() {
        let source_times = vec![0.0, 1.0, 2.0];
        let source_values = vec![0.0, 10.0, 20.0];
        let query = vec![0.5, 1.5];
        assert_eq!(interpolate_onto(&source_times, &source_values, &query), vec![5.0, 15.0]);
    }

    #[test]
    fn perfect_prediction_scores_unit_r_squared_and_positive_skill() {
        let observed = vec![1.0, 2.0, 3.0, 4.0];
        let predicted = observed.clone();
        let score = score_state("x", &predicted, &observed, 1.0);
        assert!(score.rmse.abs() < 1e-12);
        assert!((score.r_squared.unwrap() - 1.0).abs() < 1e-12);
        assert!(score.skill.unwrap() > 0.9);
    }

    #[test]
    fn verdict_flags_weak_models() {
        assert!(verdict(Some(0.1), Some(-0.5)).starts_with("WEAK"));
        assert!(verdict(Some(0.999), Some(0.9)).starts_with("STRONG"));
    }
}
