//! `lawsynth forecast` — simulate a world forward with what-if interventions,
//! optionally emitting residual-bootstrap confidence bands.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_report::{ReportOptions, UncertaintyBand, format_number, render_report};
use lawsynth_sim::{SimulationConfig, SimulationRequest, Trajectory, simulate};
use lawsynth_uncertainty::{BootstrapConfig, Samples, bootstrap, percentile};
use lawsynth_world::{VariableRole, World};

use crate::args::parse_assignment_text;
use crate::output::trajectory_csv;
use crate::read_numeric_dataset;

const DEFAULT_HORIZON: f64 = 20.0;
const DEFAULT_START: f64 = 0.0;
const DEFAULT_STEP: f64 = 0.1;
const DEFAULT_INITIAL: f64 = 1.0;
const DEFAULT_REPLICATES: usize = 512;
const DEFAULT_SEED: u64 = 0x4c53_5f46_4353_5421;
const DEFAULT_LEVEL: f64 = 0.95;

/// Help text for `lawsynth forecast`.
pub fn help() -> String {
    "lawsynth forecast WORLD.lsworld [--horizon T] [--start T] [--step DT] \
[--initial NAME=VALUE]... [--parameter NAME=VALUE]... [--intervene NAME=VALUE@TIME]... \
[--output FORECAST.csv]\n\
  Confidence bands (residual bootstrap from observed data):\n\
    [--confidence --data OBS.{csv,tsv,parquet} [--time COLUMN] [--level L] \
[--replicates N] [--seed N] [--html BANDS.html]]\n\n\
Simulates the world forward to a forecast horizon, applying constant parameter \
overrides and scheduled what-if interventions, and emits the trajectory as CSV \
plus a summary. With --confidence and --data, estimates per-state forecast spread \
by bootstrapping the model's residuals on the observed window (deterministic seed) \
and emits lower/median/upper trajectories, optionally as an HTML band report."
        .to_owned()
}

struct ForecastArgs {
    bundle: String,
    horizon: f64,
    start: f64,
    step: f64,
    initials: Vec<(Identifier, f64)>,
    parameters: Vec<(Identifier, f64)>,
    interventions: Vec<(f64, Identifier, f64)>,
    output: Option<String>,
    confidence: bool,
    data: Option<String>,
    time_column: String,
    level: f64,
    replicates: usize,
    seed: u64,
    html: Option<String>,
}

/// Runs the `forecast` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    // Fail fast on the honest confidence-without-data case, before any
    // simulation, so the user gets a clear message rather than a downstream
    // integration error.
    if args.confidence && args.data.is_none() {
        return Err("forecast --confidence needs --data OBS.csv to estimate the forecast spread; \
without observed data there is nothing to bound, so bands would be fabricated. Provide \
--data, or drop --confidence for a point forecast."
            .to_owned());
    }
    let world = read_world(&args.bundle).map_err(|error| error.to_string())?;
    let request = build_request(&world, &args)?;
    let config = SimulationConfig::new(args.start, args.horizon, args.step)
        .map_err(|error| error.to_string())?;
    let trajectory = simulate(&world, config, &request).map_err(|error| error.to_string())?;

    if args.confidence {
        return run_confidence(&world, &args, &trajectory);
    }

    let csv = trajectory_csv(&trajectory);
    let mut summary = String::new();
    if let Some(path) = &args.output {
        fs::write(path, &csv).map_err(|error| format!("failed to write {path}: {error}"))?;
        let _ = writeln!(summary, "wrote forecast: {path} ({} rows)", trajectory.samples());
    }
    let _ = writeln!(
        summary,
        "forecast horizon t in [{}, {}], {} samples",
        format_number(args.start),
        format_number(args.horizon),
        trajectory.samples()
    );
    if !args.interventions.is_empty() {
        let _ = writeln!(summary, "interventions applied: {}", args.interventions.len());
    }
    let _ = writeln!(summary, "final state:");
    for id in world.state_ids() {
        if let Some(values) = trajectory.values.get(id) {
            let first = values.first().copied().unwrap_or(f64::NAN);
            let last = values.last().copied().unwrap_or(f64::NAN);
            let _ = writeln!(
                summary,
                "  {:<16} {} -> {}",
                id.as_str(),
                format_number(first),
                format_number(last)
            );
        }
    }

    // When no --output was given, the trajectory CSV is the primary product.
    if args.output.is_none() { Ok(format!("{csv}\n{summary}")) } else { Ok(summary) }
}

/// A per-state forecast band: shared time axis plus lower/median/upper.
struct StateBand {
    state: Identifier,
    lower: Vec<f64>,
    median: Vec<f64>,
    upper: Vec<f64>,
    offset_lower: f64,
    offset_upper: f64,
    offset_error: f64,
    residuals: usize,
}

/// Estimates and emits residual-bootstrap confidence bands for the forecast.
fn run_confidence(
    world: &World,
    args: &ForecastArgs,
    forecast: &Trajectory,
) -> Result<String, String> {
    let Some(data_path) = &args.data else {
        return Err("forecast --confidence needs --data OBS.csv to estimate the forecast spread; \
without observed data there is nothing to bound, so bands would be fabricated. Provide \
--data, or drop --confidence for a point forecast."
            .to_owned());
    };
    if !(args.level.is_finite() && args.level > 0.0 && args.level < 1.0) {
        return Err("--level must be strictly between 0 and 1".to_owned());
    }
    let dataset = read_numeric_dataset(data_path, &args.time_column)?;

    // Residuals are measured by simulating the world across the observed window
    // (seeded from the first observed row), exactly the way `validate`/`monitor`
    // score a model, so the band reflects genuine model-vs-data error.
    let residuals = residuals_per_state(world, &dataset)?;
    if residuals.is_empty() {
        return Err(format!(
            "no world state matches a column in {data_path}; cannot estimate a forecast spread"
        ));
    }

    let tail = (1.0 - args.level) / 2.0;
    let config = BootstrapConfig { replicates: args.replicates, seed: args.seed };
    let mut bands = Vec::new();
    for state in world.state_ids() {
        let Some(median) = forecast.values.get(state) else { continue };
        let Some(residual) = residuals.get(state) else { continue };
        if residual.len() < 2 {
            // Too few residuals to bootstrap a spread honestly.
            continue;
        }
        let band = band_for_state(state, median, residual, config, tail)?;
        bands.push(band);
    }
    if bands.is_empty() {
        return Err(
            "every matching state had too few residuals (need >= 2) to bootstrap a band".to_owned()
        );
    }

    let csv = band_csv(&forecast.time, &bands);
    let mut summary = String::new();
    if let Some(path) = &args.output {
        fs::write(path, &csv).map_err(|error| format!("failed to write {path}: {error}"))?;
        let _ = writeln!(summary, "wrote forecast bands: {path} ({} rows)", forecast.samples());
    }
    let _ = writeln!(
        summary,
        "confidence forecast t in [{}, {}], {} samples, {:.0}% band from residual bootstrap \
({} replicates, seed {:#x})",
        format_number(args.start),
        format_number(args.horizon),
        forecast.samples(),
        args.level * 100.0,
        args.replicates,
        args.seed
    );
    let _ = writeln!(
        summary,
        "  {:<16} {:>10} {:>14} {:>14} {:>14}",
        "state", "residuals", "offset_lower", "offset_upper", "offset_se"
    );
    for band in &bands {
        let _ = writeln!(
            summary,
            "  {:<16} {:>10} {:>14} {:>14} {:>14}",
            band.state.as_str(),
            band.residuals,
            format!("{:.3e}", band.offset_lower),
            format!("{:.3e}", band.offset_upper),
            format!("{:.3e}", band.offset_error)
        );
    }

    if let Some(html_path) = &args.html {
        let html = render_band_report(world, args, &bands)?;
        fs::write(html_path, &html)
            .map_err(|error| format!("failed to write {html_path}: {error}"))?;
        let _ = writeln!(summary, "wrote band report: {html_path} ({} bytes)", html.len());
    }

    if args.output.is_none() && args.html.is_none() {
        Ok(format!("{csv}\n{summary}"))
    } else {
        Ok(summary)
    }
}

/// Bootstraps the low/high residual quantiles and builds the band trajectories.
fn band_for_state(
    state: &Identifier,
    median: &[f64],
    residual: &[f64],
    config: BootstrapConfig,
    tail: f64,
) -> Result<StateBand, String> {
    let samples = Samples::new(residual.to_vec()).map_err(|error| error.to_string())?;
    // Bootstrap the empirical quantile of the residual distribution: a deterministic,
    // seeded resample whose `observed` value is the point-estimate band offset and
    // whose spread across replicates is the offset's own standard error.
    let lower = bootstrap(&samples, config, |draw| percentile(draw, tail).unwrap_or(f64::NAN))
        .map_err(|error| error.to_string())?;
    let upper =
        bootstrap(&samples, config, |draw| percentile(draw, 1.0 - tail).unwrap_or(f64::NAN))
            .map_err(|error| error.to_string())?;
    let offset_lower = lower.observed;
    let offset_upper = upper.observed;
    let offset_error =
        lower.standard_error().unwrap_or(0.0).max(upper.standard_error().unwrap_or(0.0));
    Ok(StateBand {
        state: state.clone(),
        lower: median.iter().map(|value| value + offset_lower).collect(),
        median: median.to_vec(),
        upper: median.iter().map(|value| value + offset_upper).collect(),
        offset_lower,
        offset_upper,
        offset_error,
        residuals: residual.len(),
    })
}

/// Computes per-state residuals (observed - predicted) over the data window.
fn residuals_per_state(
    world: &World,
    dataset: &Dataset,
) -> Result<BTreeMap<Identifier, Vec<f64>>, String> {
    let times = dataset.time().values();
    if times.len() < 3 {
        return Err("need at least 3 observations to estimate residuals".to_owned());
    }
    let state_ids: Vec<Identifier> =
        world.state_ids().filter(|state| dataset.columns().contains_key(*state)).cloned().collect();
    if state_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut request = SimulationRequest::default();
    for state in &state_ids {
        request = request.with_initial(state.clone(), dataset.columns()[state].values[0]);
    }
    let step = local_step(times);
    let config = SimulationConfig::new(times[0], times[times.len() - 1], step)
        .map_err(|error| error.to_string())?;
    let trajectory = simulate(world, config, &request).map_err(|error| error.to_string())?;

    let mut residuals = BTreeMap::new();
    for state in &state_ids {
        let predicted = interpolate_onto(&trajectory.time, &trajectory.values[state], times);
        let observed = &dataset.columns()[state].values;
        let residual: Vec<f64> =
            (0..observed.len().min(predicted.len())).map(|i| observed[i] - predicted[i]).collect();
        residuals.insert(state.clone(), residual);
    }
    Ok(residuals)
}

/// Serializes the bands as CSV: `time, <state>_lower, <state>_median, <state>_upper, ...`.
fn band_csv(time: &[f64], bands: &[StateBand]) -> String {
    let mut csv = String::from("time");
    for band in bands {
        let name = band.state.as_str();
        let _ = write!(csv, ",{name}_lower,{name}_median,{name}_upper");
    }
    csv.push('\n');
    for (row, moment) in time.iter().enumerate() {
        let _ = write!(csv, "{moment:.12e}");
        for band in bands {
            let _ = write!(
                csv,
                ",{:.12e},{:.12e},{:.12e}",
                band.lower[row], band.median[row], band.upper[row]
            );
        }
        csv.push('\n');
    }
    csv
}

/// Renders the confidence bands as an HTML report via the report crate.
fn render_band_report(
    world: &World,
    args: &ForecastArgs,
    bands: &[StateBand],
) -> Result<String, String> {
    let mut options = ReportOptions {
        title: "LawSynth Forecast Confidence".to_owned(),
        start: args.start,
        end: args.horizon,
        step: args.step,
        ..Default::default()
    };
    for (id, value) in &args.initials {
        options.initial_overrides.insert(id.clone(), *value);
    }
    options.uncertainty = Some(
        bands
            .iter()
            .map(|band| UncertaintyBand {
                state: band.state.clone(),
                time: forecast_time(args),
                lower: band.lower.clone(),
                median: band.median.clone(),
                upper: band.upper.clone(),
            })
            .collect(),
    );
    render_report(world, &options).map_err(|error| error.to_string())
}

/// Reconstructs the forecast time axis from `start`, `step`, and horizon.
fn forecast_time(args: &ForecastArgs) -> Vec<f64> {
    let mut times = Vec::new();
    let mut value = args.start;
    while value <= args.horizon + args.step * 1e-9 {
        times.push(value.min(args.horizon));
        value += args.step;
    }
    times
}

fn local_step(times: &[f64]) -> f64 {
    let spacing = times[1] - times[0];
    if spacing.is_finite() && spacing > 0.0 { spacing } else { 1.0 }
}

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

fn parse(arguments: &[String]) -> Result<ForecastArgs, String> {
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle == "--help" || bundle == "-h" {
        return Err(help());
    }
    let mut args = ForecastArgs {
        bundle: bundle.clone(),
        horizon: DEFAULT_HORIZON,
        start: DEFAULT_START,
        step: DEFAULT_STEP,
        initials: Vec::new(),
        parameters: Vec::new(),
        interventions: Vec::new(),
        output: None,
        confidence: false,
        data: None,
        time_column: "time".to_owned(),
        level: DEFAULT_LEVEL,
        replicates: DEFAULT_REPLICATES,
        seed: DEFAULT_SEED,
        html: None,
    };
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--confidence" {
            args.confidence = true;
            index += 1;
            continue;
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--horizon" => args.horizon = parse_f64(value)?,
            "--start" => args.start = parse_f64(value)?,
            "--step" => args.step = parse_f64(value)?,
            "--initial" => {
                args.initials.push(parse_assignment_text(value).map_err(|e| e.to_string())?)
            }
            "--parameter" => {
                args.parameters.push(parse_assignment_text(value).map_err(|e| e.to_string())?)
            }
            "--intervene" => args.interventions.push(parse_intervention(value)?),
            "--output" => args.output = Some(value.clone()),
            "--data" => args.data = Some(value.clone()),
            "--time" => args.time_column = value.clone(),
            "--level" => args.level = parse_f64(value)?,
            "--replicates" => {
                args.replicates =
                    value.parse().map_err(|_| format!("invalid replicate count '{value}'"))?;
                if args.replicates < 2 {
                    return Err("--replicates must be at least 2".to_owned());
                }
            }
            "--seed" => {
                args.seed = value.parse().map_err(|_| format!("invalid seed '{value}'"))?;
            }
            "--html" => args.html = Some(value.clone()),
            _ => return Err(help()),
        }
        index += 2;
    }
    Ok(args)
}

fn build_request(world: &World, args: &ForecastArgs) -> Result<SimulationRequest, String> {
    let overrides: BTreeMap<Identifier, f64> = args.initials.iter().cloned().collect();
    let mut request = SimulationRequest::default();
    for state in world.state_ids() {
        let value = overrides.get(state).copied().unwrap_or(DEFAULT_INITIAL);
        request = request.with_initial(state.clone(), value);
    }
    for (id, value) in &args.parameters {
        request = request.with_parameter_override(id.clone(), *value);
    }
    for (time, id, value) in &args.interventions {
        // Route the scheduled change to a parameter or an input based on the
        // world's declarations, matching the existing simulate intervention path.
        if world.parameters().contains_key(id) {
            request = request.with_scheduled_parameter(*time, id.clone(), *value);
        } else if matches!(world.variables().get(id), Some(v) if v.role != VariableRole::State) {
            request = request.with_scheduled_input(*time, id.clone(), *value);
        } else {
            return Err(format!(
                "intervention target '{}' is neither a parameter nor a non-state input",
                id.as_str()
            ));
        }
    }
    Ok(request)
}

/// Parses the `NAME=VALUE@TIME` scheduled-intervention syntax.
fn parse_intervention(value: &str) -> Result<(f64, Identifier, f64), String> {
    let (assignment, time) =
        value.split_once('@').ok_or_else(|| "expected NAME=VALUE@TIME".to_owned())?;
    let (id, number) = parse_assignment_text(assignment).map_err(|error| error.to_string())?;
    let time = parse_f64(time)?;
    Ok((time, id, number))
}

fn parse_f64(value: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}'"))?;
    if number.is_finite() { Ok(number) } else { Err(format!("number '{value}' must be finite")) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn parses_intervention_syntax() {
        let (time, target, value) = parse_intervention("k=2.5@4").unwrap();
        assert_eq!(time, 4.0);
        assert_eq!(target, id("k"));
        assert_eq!(value, 2.5);
    }

    #[test]
    fn rejects_missing_time() {
        assert!(parse_intervention("k=2.5").is_err());
    }

    #[test]
    fn bands_bracket_the_median_and_are_deterministic() {
        let median = vec![1.0, 2.0, 3.0];
        let residual = vec![-0.5, -0.2, 0.0, 0.1, 0.4, 0.6, -0.3, 0.2];
        let config = BootstrapConfig { replicates: 128, seed: 7 };
        let first = band_for_state(&id("x"), &median, &residual, config, 0.025).unwrap();
        let second = band_for_state(&id("x"), &median, &residual, config, 0.025).unwrap();
        assert_eq!(first.lower, second.lower);
        assert_eq!(first.upper, second.upper);
        for row in 0..median.len() {
            assert!(first.lower[row] <= first.median[row]);
            assert!(first.upper[row] >= first.median[row]);
        }
    }
}
