//! `lawsynth backtest` — rigorous rolling-origin forecast evaluation.
//!
//! Where `validate` scores a single holdout, `backtest` performs walk-forward
//! validation: it picks N evenly-spaced forecast origins across the observed
//! series and, from each origin, simulates the world forward H steps and scores
//! the forecast against the actual observations. It reports per-origin skill, an
//! aggregate verdict, and how skill DECAYS with horizon (mean error at
//! h=1,2,...,H pooled across origins). Everything is deterministic — no wall
//! clock, and the same world + data + knobs yield the same report.

use std::fmt::Write as _;
use std::fs;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_report::{
    BacktestHorizonPoint, BacktestOriginRow, BacktestReport, Theme, format_number, render_backtest,
};
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::World;

use crate::read_numeric_dataset;

const DEFAULT_ORIGINS: usize = 5;
const DEFAULT_HORIZON: usize = 10;

/// Help text for `lawsynth backtest`.
pub fn help() -> String {
    "lawsynth backtest WORLD.lsworld --data OBS.{csv,tsv,parquet} [--time COLUMN] \
[--origins N] [--horizon H] [--html REPORT.html]\n\n\
Rolling-origin (walk-forward) forecast evaluation: picks N evenly-spaced forecast \
origins across the series and, from each, simulates the world forward H steps and \
scores the forecast against the actual observations. Reports per-state RMSE/MAE/R2 \
and skill vs a persistence baseline (aggregated across origins), a per-origin skill \
table, how mean error DECAYS with horizon (h=1..H), and a trust verdict. With --html \
it writes a self-contained skill-vs-horizon report.\n\n\
Defaults: --time time, --origins 5, --horizon 10."
        .to_owned()
}

struct BacktestArgs {
    bundle: String,
    data: String,
    time_column: String,
    origins: usize,
    horizon: usize,
    html: Option<String>,
}

/// Per-state forecast-skill metrics pooled across every origin and horizon.
struct StateScore {
    state: String,
    rmse: f64,
    mae: f64,
    r_squared: Option<f64>,
    skill: Option<f64>,
    samples: usize,
}

/// Aggregated error at a single forecast horizon, pooled across origins/states.
struct HorizonAggregate {
    horizon: usize,
    mean_abs_error: f64,
    rmse: f64,
}

/// One origin's per-state skill, used both for the text and HTML per-origin table.
struct OriginResult {
    index: usize,
    time: f64,
    steps: usize,
    mean_r2: Option<f64>,
    mean_skill: Option<f64>,
    mean_rmse: f64,
}

/// Runs the `backtest` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let world = read_world(&args.bundle).map_err(|error| error.to_string())?;
    let dataset = read_numeric_dataset(&args.data, &args.time_column)?;
    let evaluation = evaluate(&world, &dataset, args.origins, args.horizon)?;
    let mut report = render_text(&args, &evaluation);
    if let Some(html_path) = &args.html {
        let document = render_backtest(&build_report(&args, &evaluation));
        fs::write(html_path, &document)
            .map_err(|error| format!("failed to write {html_path}: {error}"))?;
        let _ = writeln!(report, "wrote backtest report: {html_path} ({} bytes)", document.len());
    }
    Ok(report)
}

/// The full outcome of a rolling-origin backtest.
struct Evaluation {
    origins: Vec<usize>,
    horizon: usize,
    per_state: Vec<StateScore>,
    decay: Vec<HorizonAggregate>,
    per_origin: Vec<OriginResult>,
    mean_r2: Option<f64>,
    mean_skill: Option<f64>,
}

/// Walk-forward evaluation: score forecasts from N origins over horizon H.
fn evaluate(
    world: &World,
    dataset: &Dataset,
    origins: usize,
    horizon: usize,
) -> Result<Evaluation, String> {
    let times = dataset.time().values();
    let sample_count = times.len();
    if sample_count < 4 {
        return Err("need at least 4 observations to backtest".to_owned());
    }
    if horizon == 0 {
        return Err("--horizon must be at least 1".to_owned());
    }
    if origins == 0 {
        return Err("--origins must be at least 1".to_owned());
    }
    // Every origin needs `horizon` observations ahead of it to score against.
    if sample_count < horizon + 2 {
        return Err(format!(
            "need at least {} observations for horizon {horizon} (have {sample_count})",
            horizon + 2
        ));
    }

    // The state columns must all be present in the observations.
    let state_ids: Vec<Identifier> = world.state_ids().cloned().collect();
    for state in &state_ids {
        if !dataset.columns().contains_key(state) {
            return Err(format!("state '{}' has no matching column in the data", state.as_str()));
        }
    }
    if state_ids.is_empty() {
        return Err("world has no state variables to forecast".to_owned());
    }

    let last_origin = sample_count - 1 - horizon;
    let origin_indices = evenly_spaced_origins(last_origin, origins);
    let step = integration_step(times);

    // Accumulators. Per-state pools every (origin, horizon) residual; per-horizon
    // pools every (origin, state) residual so we can show the decay curve.
    let mut state_acc: Vec<StateAccumulator> =
        state_ids.iter().map(|id| StateAccumulator::new(id.as_str())).collect();
    let mut horizon_acc: Vec<HorizonAccumulator> =
        (1..=horizon).map(HorizonAccumulator::new).collect();
    let mut per_origin = Vec::new();

    for &origin in &origin_indices {
        // Initial condition = observed state at the forecast origin.
        let mut request = SimulationRequest::default();
        for state in &state_ids {
            request = request.with_initial(state.clone(), dataset.columns()[state].values[origin]);
        }
        let start = times[origin];
        let end = times[origin + horizon];
        let config = SimulationConfig::new(start, end, step).map_err(|error| error.to_string())?;
        let trajectory = simulate(world, config, &request).map_err(|error| error.to_string())?;

        // Score this origin over horizons 1..=H.
        let query_times: Vec<f64> = (1..=horizon).map(|h| times[origin + h]).collect();
        let mut origin_state = OriginAccumulator::new();
        for (state_index, state) in state_ids.iter().enumerate() {
            let column = &dataset.columns()[state].values;
            let predicted =
                interpolate_onto(&trajectory.time, &trajectory.values[state], &query_times);
            let baseline = column[origin];
            let mut per_state_origin = ScorePool::new();
            for (h_index, h) in (1..=horizon).enumerate() {
                let observed = column[origin + h];
                let residual = predicted[h_index] - observed;
                state_acc[state_index].push(residual, observed, baseline);
                horizon_acc[h_index].push(residual);
                per_state_origin.push(residual, observed, baseline);
            }
            origin_state.add_state(per_state_origin);
        }
        per_origin.push(origin_state.finish(origin, times[origin], horizon));
    }

    let per_state: Vec<StateScore> = state_acc.iter().map(StateAccumulator::finish).collect();
    let decay: Vec<HorizonAggregate> = horizon_acc.iter().map(HorizonAccumulator::finish).collect();
    let mean_r2 = mean(&per_state.iter().filter_map(|score| score.r_squared).collect::<Vec<_>>());
    let mean_skill = mean(&per_state.iter().filter_map(|score| score.skill).collect::<Vec<_>>());

    Ok(Evaluation {
        origins: origin_indices,
        horizon,
        per_state,
        decay,
        per_origin,
        mean_r2,
        mean_skill,
    })
}

/// Picks up to `count` evenly-spaced origin indices in `[0, last_origin]`.
///
/// Deterministic and deduplicated: when the window is short, fewer distinct
/// origins are returned rather than repeating one.
fn evenly_spaced_origins(last_origin: usize, count: usize) -> Vec<usize> {
    if count <= 1 || last_origin == 0 {
        return vec![0];
    }
    let mut origins = Vec::new();
    for index in 0..count {
        // Spread inclusive of both endpoints: 0 and last_origin.
        let position = (index * last_origin) as f64 / (count - 1) as f64;
        let origin = position.round() as usize;
        if origins.last() != Some(&origin) {
            origins.push(origin);
        }
    }
    origins
}

/// Chooses a stable integration step from the series' median positive spacing.
fn integration_step(times: &[f64]) -> f64 {
    let mut diffs: Vec<f64> = times
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .filter(|delta| delta.is_finite() && *delta > 0.0)
        .collect();
    if diffs.is_empty() {
        return 1.0;
    }
    diffs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    diffs[diffs.len() / 2]
}

/// Accumulates residuals for one state across every origin and horizon.
struct StateAccumulator {
    state: String,
    pool: ScorePool,
}

impl StateAccumulator {
    fn new(state: &str) -> Self {
        Self { state: state.to_owned(), pool: ScorePool::new() }
    }

    fn push(&mut self, residual: f64, observed: f64, baseline: f64) {
        self.pool.push(residual, observed, baseline);
    }

    fn finish(&self) -> StateScore {
        let (rmse, mae, r_squared, skill) = self.pool.metrics();
        StateScore {
            state: self.state.clone(),
            rmse,
            mae,
            r_squared,
            skill,
            samples: self.pool.count,
        }
    }
}

/// Pools residuals to compute RMSE / MAE / R2 / persistence skill.
struct ScorePool {
    count: usize,
    sum_squared_error: f64,
    sum_absolute_error: f64,
    sum_observed: f64,
    sum_observed_squared: f64,
    baseline_squared_error: f64,
}

impl ScorePool {
    fn new() -> Self {
        Self {
            count: 0,
            sum_squared_error: 0.0,
            sum_absolute_error: 0.0,
            sum_observed: 0.0,
            sum_observed_squared: 0.0,
            baseline_squared_error: 0.0,
        }
    }

    fn push(&mut self, residual: f64, observed: f64, baseline: f64) {
        self.count += 1;
        self.sum_squared_error += residual * residual;
        self.sum_absolute_error += residual.abs();
        self.sum_observed += observed;
        self.sum_observed_squared += observed * observed;
        self.baseline_squared_error += (baseline - observed).powi(2);
    }

    fn metrics(&self) -> (f64, f64, Option<f64>, Option<f64>) {
        if self.count == 0 {
            return (0.0, 0.0, None, None);
        }
        let count = self.count as f64;
        let rmse = (self.sum_squared_error / count).sqrt();
        let mae = self.sum_absolute_error / count;
        // Total variance via the sum-of-squares identity (single pass).
        let mean_observed = self.sum_observed / count;
        let total_variance = self.sum_observed_squared - count * mean_observed * mean_observed;
        let r_squared = if total_variance > 0.0 {
            Some(1.0 - self.sum_squared_error / total_variance)
        } else {
            None
        };
        let skill = if self.baseline_squared_error > 0.0 {
            Some(1.0 - (self.sum_squared_error / self.baseline_squared_error).sqrt())
        } else {
            None
        };
        (rmse, mae, r_squared, skill)
    }
}

/// Accumulates absolute errors at one horizon across origins and states.
struct HorizonAccumulator {
    horizon: usize,
    count: usize,
    sum_absolute_error: f64,
    sum_squared_error: f64,
}

impl HorizonAccumulator {
    fn new(horizon: usize) -> Self {
        Self { horizon, count: 0, sum_absolute_error: 0.0, sum_squared_error: 0.0 }
    }

    fn push(&mut self, residual: f64) {
        self.count += 1;
        self.sum_absolute_error += residual.abs();
        self.sum_squared_error += residual * residual;
    }

    fn finish(&self) -> HorizonAggregate {
        let count = (self.count.max(1)) as f64;
        HorizonAggregate {
            horizon: self.horizon,
            mean_abs_error: self.sum_absolute_error / count,
            rmse: (self.sum_squared_error / count).sqrt(),
        }
    }
}

/// Accumulates a single origin's per-state pools to summarize its skill.
struct OriginAccumulator {
    states: Vec<ScorePool>,
}

impl OriginAccumulator {
    fn new() -> Self {
        Self { states: Vec::new() }
    }

    fn add_state(&mut self, pool: ScorePool) {
        self.states.push(pool);
    }

    fn finish(self, index: usize, time: f64, horizon: usize) -> OriginResult {
        let r2: Vec<f64> = self.states.iter().filter_map(|pool| pool.metrics().2).collect();
        let skill: Vec<f64> = self.states.iter().filter_map(|pool| pool.metrics().3).collect();
        let rmses: Vec<f64> = self.states.iter().map(|pool| pool.metrics().0).collect();
        OriginResult {
            index,
            time,
            steps: horizon,
            mean_r2: mean(&r2),
            mean_skill: mean(&skill),
            mean_rmse: mean(&rmses).unwrap_or(0.0),
        }
    }
}

/// Linearly interpolates `(source_times, source_values)` onto `query_times`.
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

fn render_text(args: &BacktestArgs, evaluation: &Evaluation) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Backtest: {} on {}", args.bundle, args.data);
    let _ = writeln!(
        out,
        "  rolling origin | {} origin(s) | horizon {} step(s)",
        evaluation.origins.len(),
        evaluation.horizon
    );
    out.push('\n');

    // Aggregate per-state skill (pooled across every origin and horizon).
    let _ = writeln!(
        out,
        "  {:<12} {:>12} {:>12} {:>10} {:>14} {:>8}",
        "state", "RMSE", "MAE", "R2", "skill_vs_persist", "samples"
    );
    for score in &evaluation.per_state {
        let _ = writeln!(
            out,
            "  {:<12} {:>12} {:>12} {:>10} {:>14} {:>8}",
            score.state,
            format!("{:.4e}", score.rmse),
            format!("{:.4e}", score.mae),
            score.r_squared.map(|value| format!("{value:.4}")).unwrap_or_else(|| "n/a".to_owned()),
            score.skill.map(|value| format!("{value:.4}")).unwrap_or_else(|| "n/a".to_owned()),
            score.samples,
        );
    }
    out.push('\n');

    // Skill decay with horizon.
    let _ = writeln!(out, "  skill decay with horizon (pooled across origins):");
    let _ = writeln!(out, "  {:<8} {:>14} {:>14}", "horizon", "mean|error|", "RMSE");
    for point in &evaluation.decay {
        let _ = writeln!(
            out,
            "  {:<8} {:>14} {:>14}",
            format!("h={}", point.horizon),
            format!("{:.4e}", point.mean_abs_error),
            format!("{:.4e}", point.rmse),
        );
    }
    out.push('\n');

    // Per-origin skill.
    let _ = writeln!(out, "  per-origin skill:");
    let _ = writeln!(
        out,
        "  {:<8} {:>12} {:>8} {:>10} {:>14} {:>12}",
        "origin", "t", "steps", "R2", "skill_vs_persist", "mean_RMSE"
    );
    for row in &evaluation.per_origin {
        let _ = writeln!(
            out,
            "  #{:<7} {:>12} {:>8} {:>10} {:>14} {:>12}",
            row.index,
            format_number(row.time),
            row.steps,
            row.mean_r2.map(|value| format!("{value:.4}")).unwrap_or_else(|| "n/a".to_owned()),
            row.mean_skill.map(|value| format!("{value:.4}")).unwrap_or_else(|| "n/a".to_owned()),
            format!("{:.4e}", row.mean_rmse),
        );
    }
    out.push('\n');

    if let Some(mean_r2) = evaluation.mean_r2 {
        let _ = write!(out, "  aggregate  R2={mean_r2:.4}");
        if let Some(mean_skill) = evaluation.mean_skill {
            let _ = write!(out, "  skill={mean_skill:.4}");
        }
        out.push('\n');
    }
    let _ = writeln!(out, "Verdict: {}", verdict(evaluation.mean_r2, evaluation.mean_skill));
    out
}

fn build_report(args: &BacktestArgs, evaluation: &Evaluation) -> BacktestReport {
    BacktestReport {
        title: format!("Backtest: {}", args.bundle),
        bundle_label: args.bundle.clone(),
        data_label: args.data.clone(),
        origins: evaluation.origins.len(),
        horizon: evaluation.horizon,
        decay: evaluation
            .decay
            .iter()
            .map(|point| BacktestHorizonPoint {
                horizon: point.horizon,
                mean_abs_error: point.mean_abs_error,
                rmse: point.rmse,
            })
            .collect(),
        per_origin: evaluation
            .per_origin
            .iter()
            .map(|row| BacktestOriginRow {
                origin_index: row.index,
                origin_time: row.time,
                steps: row.steps,
                mean_r2: row.mean_r2,
                mean_skill: row.mean_skill,
                mean_rmse: row.mean_rmse,
            })
            .collect(),
        verdict: verdict(evaluation.mean_r2, evaluation.mean_skill),
        theme: Theme::default(),
    }
}

fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() { None } else { Some(values.iter().sum::<f64>() / values.len() as f64) }
}

fn verdict(mean_r2: Option<f64>, mean_skill: Option<f64>) -> String {
    let Some(r2) = mean_r2 else {
        return "INCONCLUSIVE - forecast targets are constant, no variance to score".to_owned();
    };
    let beats = mean_skill.map(|skill| skill > 0.0).unwrap_or(true);
    let base = if r2 >= 0.99 && beats {
        "STRONG - forecasts stay accurate consistently across origins"
    } else if r2 >= 0.9 && beats {
        "GOOD - forecasts generalize across origins"
    } else if r2 >= 0.5 {
        "FAIR - partial forecast skill that varies across origins"
    } else {
        "WEAK - forecasts do not reliably track observations across origins"
    };
    if !beats { format!("{base} (does not beat a persistence baseline)") } else { base.to_owned() }
}

fn parse(arguments: &[String]) -> Result<BacktestArgs, String> {
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle.starts_with('-') {
        return Err(help());
    }
    let mut data = None;
    let mut time_column = "time".to_owned();
    let mut origins = DEFAULT_ORIGINS;
    let mut horizon = DEFAULT_HORIZON;
    let mut html = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--data" => data = Some(value.clone()),
            "--time" => time_column = value.clone(),
            "--origins" => {
                origins = value.parse().map_err(|_| format!("invalid origin count '{value}'"))?
            }
            "--horizon" => {
                horizon = value.parse().map_err(|_| format!("invalid horizon '{value}'"))?
            }
            "--html" => html = Some(value.clone()),
            _ => return Err(help()),
        }
        index += 2;
    }
    Ok(BacktestArgs {
        bundle: bundle.clone(),
        data: data.ok_or("missing required --data OBS.csv")?,
        time_column,
        origins,
        horizon,
        html,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origins_are_evenly_spaced_and_deduplicated() {
        assert_eq!(evenly_spaced_origins(100, 5), vec![0, 25, 50, 75, 100]);
        assert_eq!(evenly_spaced_origins(0, 5), vec![0]);
        // Short window: dedup rather than repeat.
        assert_eq!(evenly_spaced_origins(2, 5), vec![0, 1, 2]);
    }

    #[test]
    fn integration_step_is_the_median_spacing() {
        assert_eq!(integration_step(&[0.0, 1.0, 2.0, 3.0]), 1.0);
        assert_eq!(integration_step(&[0.0, 0.5, 1.0]), 0.5);
    }

    #[test]
    fn score_pool_recovers_perfect_fit() {
        let mut pool = ScorePool::new();
        // residual 0 everywhere, observed varies -> R2 = 1, positive skill.
        for observed in [1.0_f64, 2.0, 3.0, 4.0] {
            pool.push(0.0, observed, 1.0);
        }
        let (rmse, mae, r2, skill) = pool.metrics();
        assert!(rmse.abs() < 1e-12);
        assert!(mae.abs() < 1e-12);
        assert!((r2.unwrap() - 1.0).abs() < 1e-12);
        assert!(skill.unwrap() > 0.9);
    }

    #[test]
    fn verdict_flags_weak_and_strong() {
        assert!(verdict(Some(0.1), Some(-0.5)).starts_with("WEAK"));
        assert!(verdict(Some(0.999), Some(0.8)).starts_with("STRONG"));
        assert!(verdict(None, None).starts_with("INCONCLUSIVE"));
    }

    #[test]
    fn interpolates_onto_observed_grid() {
        let source_times = vec![0.0, 1.0, 2.0];
        let source_values = vec![0.0, 10.0, 20.0];
        assert_eq!(interpolate_onto(&source_times, &source_values, &[0.5, 1.5]), vec![5.0, 15.0]);
    }
}
