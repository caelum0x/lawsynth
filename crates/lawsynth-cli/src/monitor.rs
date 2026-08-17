//! `lawsynth monitor` — model-based anomaly detection.
//!
//! Simulates a world across the window of newly observed data, computes the
//! per-state, per-timestep residual (observed - predicted), standardizes each
//! residual by its own state's spread, and flags timesteps whose standardized
//! residual exceeds `K` sigma. Answers the operational question: "is my system
//! still behaving the way the discovered world says it should?"

use std::collections::BTreeSet;
use std::fmt::Write as _;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_report::format_number;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};

use crate::read_numeric_dataset;

const DEFAULT_THRESHOLD: f64 = 3.0;

/// Help text for `lawsynth monitor`.
pub fn help() -> String {
    "lawsynth monitor WORLD.lsworld --data NEW.{csv,tsv,parquet} [--time COLUMN] [--threshold K]\n\n\
Simulates the world across the new data's window (seeded from the first observed \
row), forms per-state residuals (observed - predicted), standardizes them, and \
flags timesteps whose standardized residual exceeds K sigma. Prints per-state \
residual statistics, the flagged timestamps, and an in-control / drift verdict.\n\n\
Defaults: --time time, --threshold 3."
        .to_owned()
}

struct MonitorArgs {
    bundle: String,
    data: String,
    time_column: String,
    threshold: f64,
}

/// Per-state residual diagnostics over the monitored window.
///
/// The struct and the residual/interpolation helpers below are `pub(crate)` so
/// the streaming online-discovery command (`stream`) can reuse the *identical*
/// robust standardized-residual logic instead of re-deriving it.
pub(crate) struct StateResidual {
    pub(crate) state: String,
    pub(crate) mean: f64,
    pub(crate) rms: f64,
    pub(crate) max_abs: f64,
    pub(crate) scale: f64,
    pub(crate) max_abs_z: f64,
    pub(crate) flagged: Vec<usize>,
}

/// Runs the `monitor` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let world = read_world(&args.bundle).map_err(|error| error.to_string())?;
    let dataset = read_numeric_dataset(&args.data, &args.time_column)?;

    let times = dataset.time().values();
    if times.len() < 3 {
        return Err("need at least 3 observations to monitor".to_owned());
    }
    if !(args.threshold.is_finite() && args.threshold > 0.0) {
        return Err("--threshold K must be a positive number".to_owned());
    }

    let state_ids: Vec<Identifier> = world.state_ids().cloned().collect();
    for state in &state_ids {
        if !dataset.columns().contains_key(state) {
            return Err(format!(
                "state '{}' has no matching column in {}",
                state.as_str(),
                args.data
            ));
        }
    }

    // Seed the forecast from the first observed row and integrate across the
    // whole window using the local sample spacing.
    let mut request = SimulationRequest::default();
    for state in &state_ids {
        request = request.with_initial(state.clone(), dataset.columns()[state].values[0]);
    }
    let start = times[0];
    let end = times[times.len() - 1];
    let step = local_step(times);
    let config = SimulationConfig::new(start, end, step).map_err(|error| error.to_string())?;
    let trajectory = simulate(&world, config, &request).map_err(|error| error.to_string())?;

    // Standardized residuals per state, and the union of flagged timesteps.
    let mut residuals = Vec::new();
    let mut flagged_rows: BTreeSet<usize> = BTreeSet::new();
    for state in &state_ids {
        let simulated = &trajectory.values[state];
        let predicted = interpolate_onto(&trajectory.time, simulated, times);
        let observed = &dataset.columns()[state].values;
        let residual = analyze_state(state.as_str(), &predicted, observed, args.threshold);
        for index in &residual.flagged {
            flagged_rows.insert(*index);
        }
        residuals.push(residual);
    }

    Ok(render_report(&args, times, &residuals, &flagged_rows))
}

/// Builds the residual diagnostics and flags for one state.
///
/// Flagging uses a robust control limit: residuals are centred on their median
/// and scaled by a robust spread (`1.4826 * MAD`, the normal-consistent MAD).
/// The median/MAD pair does not let a *sustained* shock inflate the limit and
/// hide itself the way a mean/std pair would. The spread is floored to a small
/// fraction of the signal's own magnitude, so a near-perfect fit whose residuals
/// sit at machine epsilon cannot be amplified into spurious anomalies.
pub(crate) fn analyze_state(
    state: &str,
    predicted: &[f64],
    observed: &[f64],
    threshold: f64,
) -> StateResidual {
    let count = observed.len().min(predicted.len());
    let residual: Vec<f64> = (0..count).map(|i| observed[i] - predicted[i]).collect();
    let mean = residual.iter().sum::<f64>() / count as f64;
    let rms = (residual.iter().map(|value| value * value).sum::<f64>() / count as f64).sqrt();
    let max_abs = residual.iter().fold(0.0_f64, |acc, value| acc.max(value.abs()));

    let median = median_of(&residual);
    let deviations: Vec<f64> = residual.iter().map(|value| (value - median).abs()).collect();
    let mad = median_of(&deviations);
    let robust_scale = 1.4826 * mad;

    // Absolute noise floor tied to the signal's magnitude: below this, residuals
    // are numerically indistinguishable from a perfect fit.
    let signal_rms = (observed.iter().map(|value| value * value).sum::<f64>()
        / observed.len().max(1) as f64)
        .sqrt();
    let floor = signal_rms.max(1.0) * 1e-9;
    let scale = robust_scale.max(floor);

    let mut max_abs_z: f64 = 0.0;
    let mut flagged = Vec::new();
    for (index, value) in residual.iter().enumerate() {
        let z = (value - median) / scale;
        max_abs_z = max_abs_z.max(z.abs());
        if z.abs() > threshold {
            flagged.push(index);
        }
    }
    StateResidual { state: state.to_owned(), mean, rms, max_abs, scale, max_abs_z, flagged }
}

/// Median of a slice via a total-order sort (empty slice yields 0).
fn median_of(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 { (sorted[mid - 1] + sorted[mid]) / 2.0 } else { sorted[mid] }
}

fn render_report(
    args: &MonitorArgs,
    times: &[f64],
    residuals: &[StateResidual],
    flagged_rows: &BTreeSet<usize>,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Monitor: {} against {}", args.bundle, args.data);
    let _ = writeln!(
        out,
        "  window t in [{}, {}] over {} observation(s), threshold K={}",
        format_number(times[0]),
        format_number(times[times.len() - 1]),
        times.len(),
        format_number(args.threshold)
    );
    out.push('\n');
    let _ = writeln!(
        out,
        "  {:<12} {:>12} {:>12} {:>12} {:>12} {:>10} {:>8}",
        "state", "mean_resid", "rms_resid", "max|resid|", "ctrl_scale", "max|z|", "flagged"
    );
    for residual in residuals {
        let _ = writeln!(
            out,
            "  {:<12} {:>12} {:>12} {:>12} {:>12} {:>10} {:>8}",
            residual.state,
            format!("{:.3e}", residual.mean),
            format!("{:.3e}", residual.rms),
            format!("{:.3e}", residual.max_abs),
            format!("{:.3e}", residual.scale),
            format!("{:.2}", residual.max_abs_z),
            residual.flagged.len()
        );
    }
    out.push('\n');

    let total_flagged = flagged_rows.len();
    if total_flagged == 0 {
        let _ = writeln!(out, "  no anomalous timesteps");
    } else {
        let listed: Vec<String> =
            flagged_rows.iter().take(12).map(|&index| format_number(times[index])).collect();
        let suffix = if total_flagged > listed.len() {
            format!(" ... (+{} more)", total_flagged - listed.len())
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "  {total_flagged} anomalous timestep(s) at t = {}{suffix}",
            listed.join(", ")
        );
    }

    let fraction = total_flagged as f64 / times.len() as f64;
    let _ = writeln!(out, "Verdict: {}", verdict(total_flagged, fraction));
    out
}

/// Chooses a fixed integration step from the local sample spacing.
pub(crate) fn local_step(times: &[f64]) -> f64 {
    let spacing = times[1] - times[0];
    if spacing.is_finite() && spacing > 0.0 { spacing } else { 1.0 }
}

/// Linearly interpolates `(source_times, source_values)` onto `query_times`,
/// clamping queries outside the source range to the nearest endpoint.
pub(crate) fn interpolate_onto(
    source_times: &[f64],
    source_values: &[f64],
    query_times: &[f64],
) -> Vec<f64> {
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

fn verdict(total_flagged: usize, fraction: f64) -> String {
    if total_flagged == 0 {
        return "IN-CONTROL - observations stay within the model's expected spread".to_owned();
    }
    if fraction >= 0.05 {
        format!(
            "DRIFT DETECTED - {:.1}% of timesteps breach the control limit; the system is no \
longer tracking the model",
            fraction * 100.0
        )
    } else {
        format!(
            "ANOMALIES FLAGGED - {:.1}% of timesteps breach the control limit; investigate the \
flagged window",
            fraction * 100.0
        )
    }
}

fn parse(arguments: &[String]) -> Result<MonitorArgs, String> {
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle.starts_with('-') {
        return Err(help());
    }
    let mut data = None;
    let mut time_column = "time".to_owned();
    let mut threshold = DEFAULT_THRESHOLD;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--data" => data = Some(value.clone()),
            "--time" => time_column = value.clone(),
            "--threshold" => {
                threshold = value.parse().map_err(|_| format!("invalid threshold '{value}'"))?;
            }
            _ => return Err(help()),
        }
        index += 2;
    }
    Ok(MonitorArgs {
        bundle: bundle.clone(),
        data: data.ok_or("missing required --data NEW.csv")?,
        time_column,
        threshold,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_match_is_in_control() {
        let observed = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let predicted = observed.clone();
        let residual = analyze_state("x", &predicted, &observed, 3.0);
        assert!(residual.flagged.is_empty());
        assert_eq!(residual.max_abs_z, 0.0);
    }

    #[test]
    fn injected_shock_is_flagged() {
        let predicted = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let mut observed = predicted.clone();
        observed[5] += 10.0; // a single large shock
        let residual = analyze_state("x", &predicted, &observed, 2.0);
        assert_eq!(residual.flagged, vec![5]);
        assert!(residual.max_abs_z > 2.0);
    }

    #[test]
    fn verdict_reports_control_state() {
        assert!(verdict(0, 0.0).starts_with("IN-CONTROL"));
        assert!(verdict(10, 0.2).starts_with("DRIFT DETECTED"));
        assert!(verdict(1, 0.01).starts_with("ANOMALIES FLAGGED"));
    }
}
