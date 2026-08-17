//! `lawsynth stream` — replayable, windowed **online** discovery (P7).
//!
//! Reads a CSV as if it streamed: it advances an explicit window across the time
//! column, keeps a *current* model, watches for a **sustained** standardized
//! residual drift (a regime/law change) as opposed to a **transient outlier**
//! (handled by `monitor`), re-discovers when a regime change is confirmed, and
//! emits an immutable change-record stream (JSONL) plus a concise summary.
//!
//! # Window policy
//!
//! One of two explicit policies over the time column (never wall clock):
//!
//! * **sliding** (default): a fixed-width window of `--window N` samples advanced
//!   by `--step M` samples.
//! * **growing** (`--growing`): an anchored window that grows by `--step M`
//!   samples from a minimum training size up to a hard cap of `--window N`
//!   samples, after which it slides at the cap.
//!
//! Ingestion reuses the bounded-memory `Read`-based delimited loader from
//! `lawsynth-data`; each window's discovery/monitoring working set tracks the
//! window, not the whole stream.
//!
//! # Determinism under replay
//!
//! Every step is deterministic and offline: no wall clock is read and no ambient
//! randomness is drawn (the default discovery path performs no resampling).
//! Replaying the identical byte stream through the identical window/config
//! produces the identical sequence of change records, byte-for-byte.
//!
//! # Honesty
//!
//! This is **not** incremental learning. Each model is re-discovered from scratch
//! over its triggering window (a batched re-run), not updated in place. The
//! change-record stream still reconstructs the full model history and replay is
//! byte-for-byte reproducible; the "online efficiency" claim is not made.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs::File;
use std::io::BufReader;
use std::ops::Range;

use lawsynth_bundle::sha256_hex;
use lawsynth_core::Identifier;
use lawsynth_data::{
    Dataset, NumericColumn, TimeAxis, load_csv_numeric_with_progress,
    load_delimited_numeric_with_progress,
};
use lawsynth_discovery::{DiscoveryCandidate, DiscoveryConfig, SparseMethod, discover};
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator, print};
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::World;

use crate::monitor::{analyze_state, interpolate_onto, local_step};
use crate::read_numeric_dataset;

const DEFAULT_WINDOW: usize = 60;
const DEFAULT_THRESHOLD: f64 = 4.0;
const DEFAULT_SUSTAIN: usize = 2;
const DEFAULT_DEGREE: usize = 2;
/// Floor on a model's fit-residual scale, as a fraction of the fitted signal's
/// own RMS. It stops a near-perfect fit (residuals at machine epsilon) from
/// amplifying ordinary rounding noise into a spurious drift.
const MIN_RELATIVE_SCALE: f64 = 1e-3;

/// Help text for `lawsynth stream`.
pub fn help() -> String {
    "lawsynth stream OBSERVATIONS.{csv,tsv,parquet} --time COLUMN --state NAME[,NAME...] \
[--window N] [--step M] [--threshold K] [--sustain W] [--degree D] [--growing] [--output HISTORY.jsonl]\n\n\
Processes the observations as if they streamed: advances a window across the time \
column, keeps a current model, and re-discovers only on a SUSTAINED standardized-\
residual drift (a regime change) over W consecutive windows -- distinct from a \
transient outlier. Every model update emits an immutable change record (prior/new \
world revision hash, the triggering window, and a per-law term/coefficient diff) as \
JSONL, plus a concise summary (windows processed, models produced, change points).\n\n\
Drift rule: a window breaches when its residual RMS under the current model \
exceeds K times that model's own fit-residual scale; re-discovery fires only when \
W consecutive windows breach (a sustained regime shift), never on a lone outlier.\n\n\
Window policy: sliding fixed-size (default) or --growing (anchored, grows by --step \
up to the --window cap, then slides). Deterministic: replaying identical bytes yields \
byte-for-byte identical change records. Honest: models are re-discovered from scratch \
over each triggering window (a batched re-run), not incrementally updated.\n\n\
Defaults: --time time, --window 60, --step = --window, --threshold 4, --sustain 2, --degree 2."
        .to_owned()
}

struct StreamArgs {
    input: String,
    time_column: String,
    state: Vec<Identifier>,
    window: usize,
    step: usize,
    threshold: f64,
    sustain: usize,
    degree: usize,
    growing: bool,
    output: Option<String>,
}

/// Runs the `stream` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h") | None) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let dataset = ingest(&args)?;
    let samples = dataset.time().len();
    for state in &args.state {
        if !dataset.columns().contains_key(state) {
            return Err(format!(
                "state '{}' has no matching column in {}",
                state.as_str(),
                args.input
            ));
        }
    }
    let min_train = min_train(args.window, args.degree);
    if samples < min_train {
        return Err(format!(
            "need at least {min_train} observations to seed a model (got {samples}); lower --window"
        ));
    }

    let ranges = window_ranges(samples, &args, min_train);
    if ranges.is_empty() {
        return Err(
            "no windows produced; check --window/--step against the sample count".to_owned()
        );
    }

    let outcome = process(&dataset, &args, &ranges)?;
    if let Some(path) = &args.output {
        std::fs::write(path, &outcome.jsonl).map_err(|error| error.to_string())?;
    }
    Ok(outcome.summary)
}

/// Reads the stream through the bounded-memory `Read`-based delimited loader.
///
/// CSV/TSV are streamed from disk in bounded chunks (peak memory tracks the
/// resulting columns, not the raw text); other formats fall back to the shared
/// numeric loader. The progress callback is intentionally silent so output stays
/// deterministic — it exists only to exercise the bounded streaming path.
fn ingest(args: &StreamArgs) -> Result<Dataset, String> {
    let lower = args.input.to_ascii_lowercase();
    let mut rows = 0usize;
    let result = if lower.ends_with(".csv") || !lower.contains('.') {
        let file = File::open(&args.input).map_err(|error| error.to_string())?;
        load_csv_numeric_with_progress(BufReader::new(file), &args.time_column, |seen| rows = seen)
    } else if lower.ends_with(".tsv") {
        let file = File::open(&args.input).map_err(|error| error.to_string())?;
        load_delimited_numeric_with_progress(
            BufReader::new(file),
            b'\t',
            &args.time_column,
            |seen| rows = seen,
        )
    } else {
        return read_numeric_dataset(&args.input, &args.time_column);
    };
    let _ = rows; // consumed by the callback; kept to document the streaming contract
    result.map_err(|error| error.to_string())
}

/// The minimum samples a window must hold before a model can be fit: a floor of
/// a few points per polynomial degree, never exceeding the configured window.
fn min_train(window: usize, degree: usize) -> usize {
    (4 * (degree + 1)).max(8).min(window)
}

/// Produces the ordered window ranges for the configured policy. Every returned
/// range holds at least `min_train` samples, so the first is a valid seed window.
fn window_ranges(samples: usize, args: &StreamArgs, min_train: usize) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    if args.growing {
        // Anchored, growing to the cap then sliding: end advances by `step`,
        // history is capped at `window`.
        let mut end = min_train.min(samples);
        loop {
            let start = end.saturating_sub(args.window);
            ranges.push(start..end);
            if end == samples {
                break;
            }
            end = (end + args.step).min(samples);
        }
    } else {
        if args.window > samples {
            return ranges;
        }
        let mut start = 0;
        while start + args.window <= samples {
            ranges.push(start..start + args.window);
            start += args.step;
        }
    }
    ranges
}

/// A discovered model plus the derived material needed to diff and hash it.
struct Model {
    world: World,
    revision: String,
    mse: f64,
    complexity: usize,
    /// Per-target additive term maps (feature string -> coefficient).
    terms: BTreeMap<String, BTreeMap<String, f64>>,
    expressions: BTreeMap<String, String>,
    /// Per-state fit-residual scale over the window the model was discovered on.
    /// A later window's residual RMS is standardized against this to detect
    /// drift; it is floored to a fraction of the signal so a perfect fit cannot
    /// amplify noise.
    scale: BTreeMap<String, f64>,
}

struct Outcome {
    summary: String,
    jsonl: String,
}

fn process(
    dataset: &Dataset,
    args: &StreamArgs,
    ranges: &[Range<usize>],
) -> Result<Outcome, String> {
    let time = dataset.time().values();
    let mut jsonl = String::new();
    let mut sequence = 0usize;
    let mut change_points: Vec<(usize, f64)> = Vec::new();

    // Seed the first model on the first window.
    let seed_range = ranges[0].clone();
    let seed_model = discover_window(dataset, args, &seed_range)?;
    jsonl.push_str(&record_json(
        sequence,
        "initial",
        None,
        &seed_model,
        &Trigger::seed(0, &seed_range, time),
        &[],
    ));
    jsonl.push('\n');
    let mut current = seed_model;
    sequence += 1;
    let mut updates = 0usize;

    let mut streak = 0usize;
    let mut windows_monitored = 0usize;
    for (index, range) in ranges.iter().enumerate().skip(1) {
        windows_monitored += 1;
        let breach = window_breach(&current, args, dataset, range);
        if breach.drift > args.threshold {
            streak += 1;
        } else {
            streak = 0;
        }
        if streak < args.sustain {
            continue;
        }
        // Sustained drift confirmed: re-discover over the triggering window.
        let candidate = discover_window(dataset, args, range)?;
        let diff = diff_models(&current, &candidate);
        let trigger = Trigger {
            window_index: index,
            rows: range.clone(),
            time_span: (time[range.start], time[range.end - 1]),
            sustained_windows: streak,
            max_abs_z: breach.max_abs_z,
            drift_ratio: breach.drift,
        };
        jsonl.push_str(&record_json(
            sequence,
            "update",
            Some(&current.revision),
            &candidate,
            &trigger,
            &diff,
        ));
        jsonl.push('\n');
        change_points.push((index, time[range.start]));
        current = candidate;
        sequence += 1;
        updates += 1;
        streak = 0;
    }

    let summary = render_summary(
        args,
        dataset.time().len(),
        ranges.len(),
        windows_monitored,
        updates,
        &change_points,
        &current,
    );
    Ok(Outcome { summary, jsonl })
}

/// Discovers a model over one window slice of the dataset.
fn discover_window(
    dataset: &Dataset,
    args: &StreamArgs,
    range: &Range<usize>,
) -> Result<Model, String> {
    let window = slice_dataset(dataset, &args.state, range)?;
    let mut config = DiscoveryConfig::new(args.state.clone());
    config.polynomial_degree = args.degree;
    config.sparse_method = SparseMethod::Stlsq;
    let result = discover(&window, &config).map_err(|error| error.to_string())?;
    let candidate = result
        .candidates
        .into_iter()
        .next()
        .ok_or_else(|| "discovery produced no candidates".to_owned())?;
    let mut model = build_model(candidate);
    // Establish the model's fit-residual scale on the window it was fit to; a
    // later window's residual RMS is standardized against this baseline.
    model.scale = residual_scales(&model.world, args, dataset, range);
    Ok(model)
}

/// Per-state fit-residual scale of `world` over `range`: the residual RMS,
/// floored to a small fraction of the signal's own RMS.
fn residual_scales(
    world: &World,
    args: &StreamArgs,
    dataset: &Dataset,
    range: &Range<usize>,
) -> BTreeMap<String, f64> {
    let residuals = residual_rms(world, args, dataset, range);
    let mut scale = BTreeMap::new();
    for state in &args.state {
        let observed = &dataset.columns()[state].values[range.clone()];
        let signal_rms = rms(observed).max(1.0);
        let floor = signal_rms * MIN_RELATIVE_SCALE;
        let fit = residuals.get(state.as_str()).copied().unwrap_or(f64::INFINITY);
        scale.insert(state.as_str().to_owned(), fit.max(floor));
    }
    scale
}

fn rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    (values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64).sqrt()
}

/// Builds an owned, aligned single-window Dataset from a row range.
fn slice_dataset(
    dataset: &Dataset,
    states: &[Identifier],
    range: &Range<usize>,
) -> Result<Dataset, String> {
    let time = TimeAxis::new(dataset.time().values()[range.clone()].to_vec())
        .map_err(|error| error.to_string())?;
    let columns = states.iter().map(|state| {
        let values = dataset.columns()[state].values[range.clone()].to_vec();
        NumericColumn::new(state.clone(), values)
    });
    Dataset::new(time, columns).map_err(|error| error.to_string())
}

/// Derives the hashable/diffable material for a discovered candidate.
fn build_model(candidate: DiscoveryCandidate) -> Model {
    let mut terms = BTreeMap::new();
    let mut expressions = BTreeMap::new();
    for (target, law) in candidate.world.laws() {
        terms.insert(target.as_str().to_owned(), additive_terms(&law.expression));
        expressions.insert(target.as_str().to_owned(), print(&law.expression));
    }
    let revision = world_revision(&candidate.world);
    Model {
        world: candidate.world,
        revision,
        mse: candidate.metrics.mean_squared_error,
        complexity: candidate.metrics.complexity,
        terms,
        expressions,
        scale: BTreeMap::new(),
    }
}

/// A deterministic, content-addressed world revision hash: SHA-256 over the
/// world's canonical declarative structure (states, parameters, and each law's
/// canonical expression, all in sorted order). Identical structure -> identical
/// hash, so the change-record stream is verifiable and replay-stable.
fn world_revision(world: &World) -> String {
    let mut canonical = String::new();
    canonical.push_str("states:");
    for id in world.state_ids() {
        let _ = write!(canonical, "{},", id.as_str());
    }
    canonical.push_str("|params:");
    for (id, parameter) in world.parameters() {
        let _ = write!(canonical, "{}={:.17e},", id.as_str(), parameter.value);
    }
    canonical.push_str("|laws:");
    for (target, law) in world.laws() {
        let _ = write!(canonical, "{}={};", target.as_str(), law.expression.to_canonical_string());
    }
    sha256_hex(canonical.as_bytes())
}

/// Per-window drift diagnostics against the current model.
struct Breach {
    /// Peak per-state standardized drift: residual RMS over the window divided by
    /// the model's fit-residual scale for that state.
    drift: f64,
    /// Peak self-standardized |z| over the window (the `monitor` statistic),
    /// reported for context.
    max_abs_z: f64,
}

/// Simulates the current world across a window, forms residuals (reusing the
/// `monitor` residual/interpolation logic), and returns the peak standardized
/// drift. A window whose model diverges under a new regime drifts to infinity.
fn window_breach(
    model: &Model,
    args: &StreamArgs,
    dataset: &Dataset,
    range: &Range<usize>,
) -> Breach {
    let residuals = residual_rms(&model.world, args, dataset, range);
    let time = &dataset.time().values()[range.clone()];
    let mut request = SimulationRequest::default();
    for state in &args.state {
        request = request.with_initial(state.clone(), dataset.columns()[state].values[range.start]);
    }
    let step = local_step(time);
    // Peak self-standardized |z| (context only), recomputed via monitor logic.
    let mut max_abs_z = 0.0_f64;
    if let Ok(config) = SimulationConfig::new(time[0], time[time.len() - 1], step) {
        if let Ok(trajectory) = simulate(&model.world, config, &request) {
            for state in &args.state {
                let predicted = interpolate_onto(&trajectory.time, &trajectory.values[state], time);
                let observed = &dataset.columns()[state].values[range.clone()];
                let residual = analyze_state(state.as_str(), &predicted, observed, args.threshold);
                max_abs_z = max_abs_z.max(residual.max_abs_z);
            }
        }
    }
    let mut drift = 0.0_f64;
    for state in &args.state {
        let scale = model.scale.get(state.as_str()).copied().unwrap_or(f64::INFINITY);
        let residual = residuals.get(state.as_str()).copied().unwrap_or(f64::INFINITY);
        let ratio = if scale > 0.0 { residual / scale } else { f64::INFINITY };
        drift = drift.max(ratio);
    }
    Breach { drift, max_abs_z }
}

/// Per-state residual RMS of `world` over `range`: seeds the simulation from the
/// window's first row, integrates across it, interpolates onto the observed grid
/// (reusing `monitor`), and returns `rms(observed - predicted)` per state. A
/// window on which the world cannot be integrated yields an infinite residual.
fn residual_rms(
    world: &World,
    args: &StreamArgs,
    dataset: &Dataset,
    range: &Range<usize>,
) -> BTreeMap<String, f64> {
    let time = &dataset.time().values()[range.clone()];
    let mut request = SimulationRequest::default();
    for state in &args.state {
        request = request.with_initial(state.clone(), dataset.columns()[state].values[range.start]);
    }
    let step = local_step(time);
    let mut scales = BTreeMap::new();
    let trajectory = SimulationConfig::new(time[0], time[time.len() - 1], step)
        .ok()
        .and_then(|config| simulate(world, config, &request).ok());
    for state in &args.state {
        let value = match &trajectory {
            Some(trajectory) => {
                let predicted = interpolate_onto(&trajectory.time, &trajectory.values[state], time);
                let observed = &dataset.columns()[state].values[range.clone()];
                let residual: Vec<f64> =
                    (0..observed.len()).map(|i| observed[i] - predicted[i]).collect();
                rms(&residual)
            }
            None => f64::INFINITY,
        };
        scales.insert(state.as_str().to_owned(), value);
    }
    scales
}

// --------------------------------------------------------------------------- //
// Term extraction and per-law diff                                            //
// --------------------------------------------------------------------------- //

/// Flattens an expression into additive `(feature -> coefficient)` terms, the
/// same view the SDK uses. Features are sorted factor products (`"x"`, `"x*y"`,
/// `"1"` for a constant); unsupported factors (trig/rational) become an opaque,
/// stable printed feature so a term-level diff is always well defined.
fn additive_terms(expression: &Expr) -> BTreeMap<String, f64> {
    let mut map = BTreeMap::new();
    collect_terms(expression, 1.0, &mut map);
    map.retain(|_, coefficient| coefficient.abs() > 0.0);
    map
}

fn collect_terms(expression: &Expr, sign: f64, map: &mut BTreeMap<String, f64>) {
    match expression {
        Expr::Binary { operator: BinaryOperator::Add, left, right } => {
            collect_terms(left, sign, map);
            collect_terms(right, sign, map);
        }
        Expr::Binary { operator: BinaryOperator::Subtract, left, right } => {
            collect_terms(left, sign, map);
            collect_terms(right, -sign, map);
        }
        Expr::Unary { operator: UnaryOperator::Negate, operand } => {
            collect_terms(operand, -sign, map);
        }
        other => {
            let mut coefficient = sign;
            let mut factors: Vec<String> = Vec::new();
            product_of(other, &mut coefficient, &mut factors);
            *map.entry(feature_key(&mut factors)).or_insert(0.0) += coefficient;
        }
    }
}

fn product_of(expression: &Expr, coefficient: &mut f64, factors: &mut Vec<String>) {
    match expression {
        Expr::Constant(value) => *coefficient *= value,
        Expr::Symbol(id) => factors.push(id.as_str().to_owned()),
        Expr::Unary { operator: UnaryOperator::Negate, operand } => {
            *coefficient *= -1.0;
            product_of(operand, coefficient, factors);
        }
        Expr::Binary { operator: BinaryOperator::Multiply, left, right } => {
            product_of(left, coefficient, factors);
            product_of(right, coefficient, factors);
        }
        Expr::Binary { operator: BinaryOperator::Divide, left, right } => {
            if let Expr::Constant(value) = right.as_ref() {
                product_of(left, coefficient, factors);
                if *value != 0.0 {
                    *coefficient /= value;
                }
            } else {
                factors.push(print(expression));
            }
        }
        Expr::Binary { operator: BinaryOperator::Power, left, right } => {
            if let (Expr::Symbol(id), Expr::Constant(power)) = (left.as_ref(), right.as_ref()) {
                let repeats = if *power >= 1.0 { *power as usize } else { 0 };
                for _ in 0..repeats {
                    factors.push(id.as_str().to_owned());
                }
                if repeats == 0 {
                    factors.push(print(expression));
                }
            } else {
                factors.push(print(expression));
            }
        }
        other => factors.push(print(other)),
    }
}

fn feature_key(factors: &mut [String]) -> String {
    if factors.is_empty() {
        return "1".to_owned();
    }
    factors.sort();
    factors.join("*")
}

/// One per-law coefficient change in a model transition.
struct TermChange {
    target: String,
    feature: String,
    prior: f64,
    new: f64,
    kind: &'static str,
}

/// Diffs two models law-by-law, reporting every feature whose coefficient moved.
fn diff_models(prior: &Model, new: &Model) -> Vec<TermChange> {
    let mut changes = Vec::new();
    let targets: BTreeSet<&String> = prior.terms.keys().chain(new.terms.keys()).collect();
    for target in targets {
        let empty = BTreeMap::new();
        let prior_terms = prior.terms.get(target).unwrap_or(&empty);
        let new_terms = new.terms.get(target).unwrap_or(&empty);
        let features: BTreeSet<&String> = prior_terms.keys().chain(new_terms.keys()).collect();
        for feature in features {
            let before = prior_terms.get(feature).copied().unwrap_or(0.0);
            let after = new_terms.get(feature).copied().unwrap_or(0.0);
            let tolerance = 1e-9 * (1.0 + before.abs().max(after.abs()));
            if (before - after).abs() <= tolerance {
                continue;
            }
            let kind = if before == 0.0 {
                "added"
            } else if after == 0.0 {
                "removed"
            } else {
                "changed"
            };
            changes.push(TermChange {
                target: target.clone(),
                feature: feature.clone(),
                prior: before,
                new: after,
                kind,
            });
        }
    }
    changes
}

// --------------------------------------------------------------------------- //
// JSON serialisation (hand-rolled — std-only, no external crates)             //
// --------------------------------------------------------------------------- //

struct Trigger {
    window_index: usize,
    rows: Range<usize>,
    time_span: (f64, f64),
    sustained_windows: usize,
    max_abs_z: f64,
    drift_ratio: f64,
}

impl Trigger {
    fn seed(index: usize, range: &Range<usize>, time: &[f64]) -> Self {
        Self {
            window_index: index,
            rows: range.clone(),
            time_span: (time[range.start], time[range.end - 1]),
            sustained_windows: 0,
            max_abs_z: 0.0,
            drift_ratio: 0.0,
        }
    }
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn number(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.17e}")
    } else if value == f64::INFINITY {
        "\"Infinity\"".to_owned()
    } else if value == f64::NEG_INFINITY {
        "\"-Infinity\"".to_owned()
    } else {
        "\"NaN\"".to_owned()
    }
}

fn record_json(
    sequence: usize,
    kind: &str,
    prior_revision: Option<&str>,
    model: &Model,
    trigger: &Trigger,
    diff: &[TermChange],
) -> String {
    let mut out = String::new();
    out.push('{');
    let _ = write!(out, "\"sequence\":{sequence},");
    let _ = write!(out, "\"kind\":\"{}\",", json_escape(kind));
    match prior_revision {
        Some(revision) => {
            let _ = write!(out, "\"prior_revision\":\"{}\",", json_escape(revision));
        }
        None => out.push_str("\"prior_revision\":null,"),
    }
    let _ = write!(out, "\"new_revision\":\"{}\",", json_escape(&model.revision));
    // trigger
    let _ = write!(
        out,
        "\"trigger\":{{\"window_index\":{},\"rows\":[{},{}],\"time_span\":[{},{}],\
\"sustained_windows\":{},\"max_abs_z\":{},\"drift_ratio\":{}}},",
        trigger.window_index,
        trigger.rows.start,
        trigger.rows.end,
        number(trigger.time_span.0),
        number(trigger.time_span.1),
        trigger.sustained_windows,
        number(trigger.max_abs_z),
        number(trigger.drift_ratio),
    );
    // metrics
    let _ = write!(
        out,
        "\"metrics\":{{\"mse\":{},\"complexity\":{}}},",
        number(model.mse),
        model.complexity
    );
    // laws
    out.push_str("\"laws\":[");
    for (i, (target, expression)) in model.expressions.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&law_json(target, expression, &model.terms[target]));
    }
    out.push_str("],");
    // diff
    out.push_str("\"diff\":[");
    for (i, change) in diff.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{{\"target\":\"{}\",\"feature\":\"{}\",\"kind\":\"{}\",\"prior\":{},\"new\":{}}}",
            json_escape(&change.target),
            json_escape(&change.feature),
            change.kind,
            number(change.prior),
            number(change.new),
        );
    }
    out.push(']');
    out.push('}');
    out
}

fn law_json(target: &str, expression: &str, terms: &BTreeMap<String, f64>) -> String {
    // Order terms by descending magnitude, then feature, for a stable reading.
    let mut ordered: Vec<(&String, &f64)> = terms.iter().collect();
    ordered.sort_by(|a, b| {
        b.1.abs().partial_cmp(&a.1.abs()).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(b.0))
    });
    let mut out = String::new();
    let _ = write!(
        out,
        "{{\"target\":\"{}\",\"expression\":\"{}\",\"terms\":[",
        json_escape(target),
        json_escape(expression)
    );
    for (i, (feature, coefficient)) in ordered.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{{\"feature\":\"{}\",\"coefficient\":{}}}",
            json_escape(feature),
            number(**coefficient)
        );
    }
    out.push_str("]}");
    out
}

// --------------------------------------------------------------------------- //
// Summary                                                                     //
// --------------------------------------------------------------------------- //

#[allow(clippy::too_many_arguments)]
fn render_summary(
    args: &StreamArgs,
    samples: usize,
    windows: usize,
    monitored: usize,
    updates: usize,
    change_points: &[(usize, f64)],
    current: &Model,
) -> String {
    let policy = if args.growing {
        format!("growing (grows by {} to a cap of {} samples)", args.step, args.window)
    } else {
        format!("sliding (width {} step {})", args.window, args.step)
    };
    let mut out = String::new();
    let _ = writeln!(out, "Stream: {} ({} samples)", args.input, samples);
    let _ = writeln!(
        out,
        "  policy: {policy}; threshold K={}sigma; sustain {} consecutive window(s)",
        args.threshold, args.sustain
    );
    let _ = writeln!(out, "  windows processed: {windows} ({monitored} monitored after the seed)");
    let _ =
        writeln!(out, "  models produced: {} (1 initial + {updates} re-discovery)", updates + 1);
    if change_points.is_empty() {
        let _ = writeln!(out, "  change points: none (dynamics stable across the stream)");
    } else {
        let listed: Vec<String> = change_points
            .iter()
            .map(|(index, time)| format!("window {index} (t={time:.4})"))
            .collect();
        let _ = writeln!(out, "  change points: {}", listed.join(", "));
    }
    let _ = writeln!(
        out,
        "  final model revision: {}",
        &current.revision[..16.min(current.revision.len())]
    );
    for (target, expression) in &current.expressions {
        let _ = writeln!(out, "    d{target}/dt = {expression}");
    }
    if let Some(path) = &args.output {
        let _ = writeln!(out, "  change-record history -> {path}");
    }
    out.push_str(
        "  NOTE: each model is re-discovered from scratch over its triggering window \
(a batched re-run), not incrementally updated.\n",
    );
    out
}

// --------------------------------------------------------------------------- //
// Argument parsing                                                            //
// --------------------------------------------------------------------------- //

fn parse(arguments: &[String]) -> Result<StreamArgs, String> {
    let Some(input) = arguments.first() else {
        return Err(help());
    };
    if input.starts_with('-') {
        return Err(help());
    }
    let mut time_column = "time".to_owned();
    let mut state = None;
    let mut window = None;
    let mut step = None;
    let mut threshold = DEFAULT_THRESHOLD;
    let mut sustain = DEFAULT_SUSTAIN;
    let mut degree = DEFAULT_DEGREE;
    let mut growing = false;
    let mut output = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--growing" {
            growing = true;
            index += 1;
            continue;
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--time" => time_column = value.clone(),
            "--state" => state = Some(parse_identifiers(value)?),
            "--window" => window = Some(parse_usize(value, "--window")?),
            "--step" => step = Some(parse_usize(value, "--step")?),
            "--threshold" => {
                threshold = value.parse().map_err(|_| format!("invalid threshold '{value}'"))?;
                if !(threshold.is_finite() && threshold > 0.0) {
                    return Err("--threshold K must be a positive number".to_owned());
                }
            }
            "--sustain" => {
                sustain = parse_usize(value, "--sustain")?;
                if sustain == 0 {
                    return Err("--sustain must be at least 1".to_owned());
                }
            }
            "--degree" => degree = parse_usize(value, "--degree")?,
            "--output" => output = Some(value.clone()),
            _ => return Err(help()),
        }
        index += 2;
    }
    let window = window.unwrap_or(DEFAULT_WINDOW);
    if window == 0 {
        return Err("--window must be at least 1".to_owned());
    }
    let step = step.unwrap_or(window);
    if step == 0 {
        return Err("--step must be at least 1".to_owned());
    }
    Ok(StreamArgs {
        input: input.clone(),
        time_column,
        state: state.ok_or("missing required --state NAME[,NAME...]")?,
        window,
        step,
        threshold,
        sustain,
        degree,
        growing,
        output,
    })
}

fn parse_identifiers(value: &str) -> Result<Vec<Identifier>, String> {
    let identifiers = value
        .split(',')
        .map(|item| Identifier::new(item.trim()).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if identifiers.is_empty() {
        Err("expected at least one state identifier".to_owned())
    } else {
        Ok(identifiers)
    }
}

fn parse_usize(value: &str, flag: &str) -> Result<usize, String> {
    value.parse().map_err(|_| format!("invalid {flag} value '{value}'"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn sample_args() -> StreamArgs {
        StreamArgs {
            input: "in.csv".to_owned(),
            time_column: "time".to_owned(),
            state: vec![id("x")],
            window: 10,
            step: 10,
            threshold: 3.0,
            sustain: 2,
            degree: 2,
            growing: false,
            output: None,
        }
    }

    #[test]
    fn sliding_windows_are_aligned_and_stepped() {
        let args = sample_args();
        let ranges = window_ranges(30, &args, 8);
        assert_eq!(ranges, vec![0..10, 10..20, 20..30]);
    }

    #[test]
    fn growing_windows_grow_to_the_cap_then_slide() {
        let mut args = sample_args();
        args.growing = true;
        args.window = 10;
        args.step = 5;
        let ranges = window_ranges(25, &args, 8);
        // Seeds at min_train=8, grows to the cap of 10, then slides by 5.
        assert_eq!(ranges.first().unwrap().len(), 8);
        assert!(ranges.iter().all(|range| range.len() <= 10));
        assert_eq!(ranges.last().unwrap().end, 25);
    }

    #[test]
    fn additive_terms_flatten_a_linear_law() {
        // -0.3*y + 0.5*x
        let expression = Expr::sum(
            Expr::product(Expr::constant(-0.3), Expr::symbol(id("y"))),
            Expr::product(Expr::constant(0.5), Expr::symbol(id("x"))),
        );
        let terms = additive_terms(&expression);
        assert!((terms["y"] + 0.3).abs() < 1e-12);
        assert!((terms["x"] - 0.5).abs() < 1e-12);
    }

    #[test]
    fn diff_names_changed_terms() {
        let prior = model_with(id("x"), Expr::product(Expr::constant(1.0), Expr::symbol(id("x"))));
        let new = model_with(id("x"), Expr::product(Expr::constant(-2.0), Expr::symbol(id("x"))));
        let changes = diff_models(&prior, &new);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].feature, "x");
        assert_eq!(changes[0].kind, "changed");
    }

    fn model_with(target: Identifier, expression: Expr) -> Model {
        let mut terms = BTreeMap::new();
        terms.insert(target.as_str().to_owned(), additive_terms(&expression));
        let mut expressions = BTreeMap::new();
        expressions.insert(target.as_str().to_owned(), print(&expression));
        // A dummy world is not needed for the diff test; reuse a minimal build.
        let world = World::new(
            [lawsynth_world::Variable::new(target.clone(), lawsynth_world::VariableRole::State)],
            [],
            [lawsynth_world::ContinuousLaw::new(target, expression)],
        )
        .unwrap();
        let revision = world_revision(&world);
        Model {
            world,
            revision,
            mse: 0.0,
            complexity: 0,
            terms,
            expressions,
            scale: BTreeMap::new(),
        }
    }
}
