//! `lawsynth pipeline` — a reproducible, declarative end-to-end workflow.
//!
//! One config file drives the whole engine: ingest a CSV, discover a law
//! system, optionally validate on a holdout, then render a report and export
//! artifacts. Everything is deterministic and offline, so the same config and
//! data always reproduce the same worlds, reports, and summary.
//!
//! The config is parsed with a small hand-rolled reader (sections plus
//! `key = value` lines, with `#`/`;` comments and `[...]` arrays) — no external
//! TOML crate is used.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;

use lawsynth_bundle::write_world;
use lawsynth_core::Identifier;
use lawsynth_discovery::{DiscoveryConfig, SparseMethod, discover};
use lawsynth_report::{
    RegimeSpan, ReportObservations, ReportOptions, format_number, render_report,
};

use crate::{export, read_numeric_dataset, validate};

/// Help text for `lawsynth pipeline`.
pub fn help() -> String {
    "lawsynth pipeline <pipeline.toml>\n  lawsynth pipeline --example\n\n\
Runs a reproducible, declarative workflow from one config file: ingest a CSV, \
discover a law system, optionally validate on a holdout, then write a .lsworld \
bundle, an HTML report, and optional python/latex exports. Deterministic and \
offline.\n\n\
Use --example to print a documented sample config."
        .to_owned()
}

/// Runs the `pipeline` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    match arguments.first().map(String::as_str) {
        None => Err(help()),
        Some("--help" | "-h") => Ok(help()),
        Some("--example") => Ok(example_config()),
        Some(path) => run_config(path),
    }
}

fn run_config(path: &str) -> Result<String, String> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    let config = Config::parse(&text)?;
    let plan = PipelinePlan::from_config(&config)?;
    execute(&plan)
}

// --- Config model ---------------------------------------------------------

/// Everything a pipeline run needs, resolved from the config file.
struct PipelinePlan {
    csv: String,
    time_column: String,
    state: Vec<Identifier>,
    discovery: DiscoveryConfig,
    report_pareto: bool,
    validate_holdout: Option<f64>,
    world_output: String,
    report_output: Option<String>,
    report_title: String,
    export_python: Option<String>,
    export_latex: Option<String>,
}

impl PipelinePlan {
    fn from_config(config: &Config) -> Result<Self, String> {
        let csv = config.require_str("input", "csv")?;
        let time_column = config.string("input", "time").unwrap_or_else(|| "time".to_owned());
        let state_names = config.array("input", "state").ok_or_else(|| {
            "config error: [input] must declare `state = [\"NAME\", ...]`".to_owned()
        })?;
        if state_names.is_empty() {
            return Err("config error: [input] state must list at least one column".to_owned());
        }
        let state: Vec<Identifier> = state_names
            .iter()
            .map(|name| Identifier::new(name).map_err(|error| error.to_string()))
            .collect::<Result<_, _>>()?;

        let mut discovery = DiscoveryConfig::new(state.clone());
        if let Some(degree) = config.usize_value("discovery", "degree")? {
            discovery.polynomial_degree = degree;
        }
        if let Some(threshold) = config.f64_value("discovery", "threshold")? {
            discovery.sparse.threshold = threshold;
        }
        if let Some(solver) = config.string("discovery", "solver") {
            discovery.sparse_method = match solver.as_str() {
                "stlsq" => SparseMethod::Stlsq,
                "sr3" => SparseMethod::Sr3,
                other => {
                    return Err(format!("config error: solver '{other}' must be 'stlsq' or 'sr3'"));
                }
            };
        }
        discovery.include_trigonometric = config.bool_value("discovery", "trigonometric")?;
        discovery.include_rational = config.bool_value("discovery", "rational")?;
        if config.bool_value("discovery", "regimes")? {
            discovery.enable_regimes();
        }
        if config.bool_value("discovery", "refine")? {
            discovery.enable_refinement();
        }
        if config.bool_value("discovery", "causal")? {
            discovery.enable_causal_hypothesis();
        }
        let report_pareto = config.bool_value("discovery", "pareto")?;

        let validate_holdout = config.f64_value("validate", "holdout")?;

        let world_output = config.require_str("outputs", "world")?;
        let report_output = config.string("outputs", "report");
        let report_title = config
            .string("outputs", "title")
            .unwrap_or_else(|| format!("LawSynth pipeline: {world_output}"));
        let export_python = config.string("outputs", "export_python");
        let export_latex = config.string("outputs", "export_latex");

        Ok(Self {
            csv,
            time_column,
            state,
            discovery,
            report_pareto,
            validate_holdout,
            world_output,
            report_output,
            report_title,
            export_python,
            export_latex,
        })
    }
}

// --- Execution ------------------------------------------------------------

fn execute(plan: &PipelinePlan) -> Result<String, String> {
    let dataset = read_numeric_dataset(&plan.csv, &plan.time_column)?;

    // Ingest -> discover.
    let result = discover(&dataset, &plan.discovery).map_err(|error| error.to_string())?;
    let frontier_size = result.frontier.len();
    // Convert any discovered segmentation into report regime spans. Field access
    // only, so the CLI need not depend on `lawsynth-regime` directly.
    let regime_spans: Option<Vec<RegimeSpan>> = result.regimes.as_ref().map(|segmentation| {
        segmentation
            .segments
            .iter()
            .map(|segment| RegimeSpan {
                start: segment.start,
                end: segment.end,
                label: format_number(segment.mean),
            })
            .collect()
    });
    let candidate = result
        .candidates
        .into_iter()
        .next()
        .ok_or_else(|| "discovery produced no candidates".to_owned())?;
    let world = candidate.world;
    let mse = candidate.metrics.mean_squared_error;
    let complexity = candidate.metrics.complexity;

    // Write the world bundle.
    write_world(&plan.world_output, &world).map_err(|error| error.to_string())?;
    let mut artifacts: Vec<String> = vec![plan.world_output.clone()];

    // Optional validation on a time holdout.
    let mut verdict = None;
    if let Some(holdout) = plan.validate_holdout {
        let summary =
            validate::validate_dataset(&world, &dataset, holdout, &plan.world_output, &plan.csv)?;
        verdict = Some(summary.verdict);
    }

    // Report: overlay the observations and any discovered regimes.
    if let Some(report_path) = &plan.report_output {
        let options = build_report_options(plan, &world, &dataset, regime_spans.clone());
        let html = render_report(&world, &options).map_err(|error| error.to_string())?;
        fs::write(report_path, &html)
            .map_err(|error| format!("failed to write {report_path}: {error}"))?;
        artifacts.push(report_path.clone());
    }

    // Optional exports.
    let stem = std::path::Path::new(&plan.world_output)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("world");
    if let Some(python_path) = &plan.export_python {
        let module = export::emit_python(&world, stem);
        fs::write(python_path, &module)
            .map_err(|error| format!("failed to write {python_path}: {error}"))?;
        artifacts.push(python_path.clone());
    }
    if let Some(latex_path) = &plan.export_latex {
        let latex = export::emit_latex(&world, stem);
        fs::write(latex_path, &latex)
            .map_err(|error| format!("failed to write {latex_path}: {error}"))?;
        artifacts.push(latex_path.clone());
    }

    Ok(render_summary(plan, mse, complexity, frontier_size, verdict.as_deref(), &artifacts))
}

/// Builds report options aligned to the observations so residuals are meaningful.
fn build_report_options(
    plan: &PipelinePlan,
    world: &lawsynth_world::World,
    dataset: &lawsynth_data::Dataset,
    regime_spans: Option<Vec<RegimeSpan>>,
) -> ReportOptions {
    let mut options =
        ReportOptions { title: plan.report_title.clone(), ..ReportOptions::default() };
    let times = dataset.time().values();
    if times.len() >= 2 {
        options.start = times[0];
        options.end = times[times.len() - 1];
        let spacing = times[1] - times[0];
        options.step = if spacing.is_finite() && spacing > 0.0 { spacing } else { 0.1 };
    }
    let mut columns = BTreeMap::new();
    for state in world.state_ids() {
        if let Some(column) = dataset.columns().get(state) {
            options.initial_overrides.insert(state.clone(), column.values[0]);
            columns.insert(state.clone(), column.values.clone());
        }
    }
    if !columns.is_empty() {
        options.observations = Some(ReportObservations { time: times.to_vec(), columns });
    }
    if let Some(spans) = regime_spans {
        if !spans.is_empty() {
            options.regimes = Some(spans);
        }
    }
    options
}

fn render_summary(
    plan: &PipelinePlan,
    mse: f64,
    complexity: usize,
    frontier_size: usize,
    verdict: Option<&str>,
    artifacts: &[String],
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "pipeline: {}", plan.csv);
    let _ = writeln!(
        out,
        "  discovered {} state law(s): mse={mse:.6e}, complexity={complexity}",
        plan.state.len()
    );
    if plan.report_pareto {
        let _ = writeln!(out, "  pareto frontier: {frontier_size} candidate(s)");
    }
    match verdict {
        Some(verdict) => {
            let _ = writeln!(out, "  validate: {verdict}");
        }
        None => {
            let _ = writeln!(out, "  validate: skipped (no [validate] section)");
        }
    }
    let _ = writeln!(out, "  artifacts:");
    for artifact in artifacts {
        let _ = writeln!(out, "    - {artifact}");
    }
    out
}

/// A documented, ready-to-run sample config printed by `pipeline --example`.
fn example_config() -> String {
    "# LawSynth pipeline config (hand-rolled TOML-ish: sections + key = value).\n\
# Run with:  lawsynth pipeline pipeline.toml\n\
\n\
[input]\n\
csv = \"observations.csv\"   # ingest this CSV\n\
time = \"time\"               # name of the time column\n\
state = [\"x\", \"y\"]          # state columns to discover laws for\n\
\n\
[discovery]\n\
degree = 2                  # polynomial feature degree\n\
threshold = 0.05            # sparse coefficient threshold\n\
solver = \"stlsq\"            # stlsq | sr3\n\
trigonometric = false       # add sin/cos features\n\
rational = false            # add rational features\n\
regimes = false             # segment the primary state into regimes\n\
pareto = false              # report Pareto frontier size\n\
refine = false              # joint parameter refinement\n\
causal = false              # dependency/causal hypothesis\n\
\n\
[validate]                  # optional: omit to skip validation\n\
holdout = 0.2               # fraction held out (by time) to score forecast skill\n\
\n\
[outputs]\n\
world = \"model.lsworld\"     # required: the discovered world bundle\n\
report = \"model.report.html\" # self-contained HTML report (with residuals)\n\
title = \"My model\"          # optional report title\n\
export_python = \"model.py\"  # optional: runnable python module\n\
export_latex = \"model.tex\"  # optional: LaTeX law system\n"
        .to_owned()
}

// --- Hand-rolled config reader -------------------------------------------

/// A parsed value: either a scalar token or an array of tokens.
enum Value {
    Scalar(String),
    Array(Vec<String>),
}

/// Sections mapping to `key -> Value`.
struct Config {
    sections: BTreeMap<String, BTreeMap<String, Value>>,
}

impl Config {
    fn parse(text: &str) -> Result<Self, String> {
        let mut sections: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
        let mut current = String::new();
        for (number, raw) in text.lines().enumerate() {
            let line = strip_comment(raw).trim();
            if line.is_empty() {
                continue;
            }
            if let Some(inner) = line.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
                current = inner.trim().to_owned();
                sections.entry(current.clone()).or_default();
                continue;
            }
            let (key, value) = line.split_once('=').ok_or_else(|| {
                format!("config error on line {}: expected `key = value`", number + 1)
            })?;
            let key = key.trim().to_owned();
            if key.is_empty() {
                return Err(format!("config error on line {}: empty key", number + 1));
            }
            let value = parse_value(value.trim());
            sections.entry(current.clone()).or_default().insert(key, value);
        }
        Ok(Self { sections })
    }

    fn get(&self, section: &str, key: &str) -> Option<&Value> {
        self.sections.get(section).and_then(|keys| keys.get(key))
    }

    fn string(&self, section: &str, key: &str) -> Option<String> {
        match self.get(section, key) {
            Some(Value::Scalar(value)) => Some(value.clone()),
            _ => None,
        }
    }

    fn require_str(&self, section: &str, key: &str) -> Result<String, String> {
        self.string(section, key)
            .ok_or_else(|| format!("config error: [{section}] must set `{key} = \"...\"`"))
    }

    fn array(&self, section: &str, key: &str) -> Option<Vec<String>> {
        match self.get(section, key) {
            Some(Value::Array(values)) => Some(values.clone()),
            // Accept a bare comma-separated scalar as a convenience.
            Some(Value::Scalar(value)) => {
                Some(value.split(',').map(|item| item.trim().to_owned()).collect())
            }
            None => None,
        }
    }

    fn f64_value(&self, section: &str, key: &str) -> Result<Option<f64>, String> {
        match self.string(section, key) {
            None => Ok(None),
            Some(value) => {
                let number: f64 = value.parse().map_err(|_| {
                    format!("config error: [{section}] {key}='{value}' is not a number")
                })?;
                if !number.is_finite() {
                    return Err(format!("config error: [{section}] {key} must be finite"));
                }
                Ok(Some(number))
            }
        }
    }

    fn usize_value(&self, section: &str, key: &str) -> Result<Option<usize>, String> {
        match self.string(section, key) {
            None => Ok(None),
            Some(value) => value.parse().map(Some).map_err(|_| {
                format!("config error: [{section}] {key}='{value}' is not an integer")
            }),
        }
    }

    fn bool_value(&self, section: &str, key: &str) -> Result<bool, String> {
        match self.string(section, key) {
            None => Ok(false),
            Some(value) => match value.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                other => {
                    Err(format!("config error: [{section}] {key}='{other}' must be true or false"))
                }
            },
        }
    }
}

/// Removes an inline `#` or `;` comment that is not inside a quoted string.
fn strip_comment(line: &str) -> &str {
    let mut in_quotes = false;
    for (index, character) in line.char_indices() {
        match character {
            '"' => in_quotes = !in_quotes,
            '#' | ';' if !in_quotes => return &line[..index],
            _ => {}
        }
    }
    line
}

/// Parses a scalar or `[a, b, c]` array, unquoting `"..."` tokens.
fn parse_value(raw: &str) -> Value {
    if let Some(inner) = raw.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
        let items = inner
            .split(',')
            .map(|item| unquote(item.trim()))
            .filter(|item| !item.is_empty())
            .collect();
        Value::Array(items)
    } else {
        Value::Scalar(unquote(raw))
    }
}

/// Strips a single pair of surrounding double quotes if present.
fn unquote(token: &str) -> String {
    let trimmed = token.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_keys_and_arrays() {
        let text = "# comment\n\
            [input]\n\
            csv = \"obs.csv\"  # inline comment\n\
            state = [\"x\", \"y\"]\n\
            [discovery]\n\
            degree = 3\n\
            regimes = true\n";
        let config = Config::parse(text).unwrap();
        assert_eq!(config.string("input", "csv").as_deref(), Some("obs.csv"));
        assert_eq!(config.array("input", "state").unwrap(), vec!["x", "y"]);
        assert_eq!(config.usize_value("discovery", "degree").unwrap(), Some(3));
        assert!(config.bool_value("discovery", "regimes").unwrap());
        assert!(!config.bool_value("discovery", "trigonometric").unwrap());
    }

    #[test]
    fn strips_comments_outside_quotes_only() {
        assert_eq!(strip_comment("a = \"x # y\" # tail").trim(), "a = \"x # y\"");
        assert_eq!(strip_comment("; whole line").trim(), "");
    }

    #[test]
    fn plan_requires_input_and_outputs() {
        let text = "[input]\ncsv = \"o.csv\"\nstate = [\"x\"]\n[outputs]\nworld = \"w.lsworld\"\n";
        let config = Config::parse(text).unwrap();
        let plan = PipelinePlan::from_config(&config).unwrap();
        assert_eq!(plan.csv, "o.csv");
        assert_eq!(plan.world_output, "w.lsworld");
        assert_eq!(plan.state.len(), 1);
        assert!(plan.validate_holdout.is_none());
    }

    #[test]
    fn missing_state_is_an_error() {
        let text = "[input]\ncsv = \"o.csv\"\n[outputs]\nworld = \"w.lsworld\"\n";
        let config = Config::parse(text).unwrap();
        assert!(PipelinePlan::from_config(&config).is_err());
    }

    #[test]
    fn example_config_round_trips_into_a_plan() {
        let config = Config::parse(&example_config()).unwrap();
        let plan = PipelinePlan::from_config(&config).unwrap();
        assert_eq!(plan.state.len(), 2);
        assert_eq!(plan.validate_holdout, Some(0.2));
        assert!(plan.report_output.is_some());
        assert!(plan.export_python.is_some());
    }
}
