//! Command implementation shared by the LawSynth binary and integration tests.

mod args;
mod backtest;
mod compare;
mod compose;
mod config;
mod discover;
mod doctor;
mod edit;
mod error;
mod explain;
mod export;
mod forecast;
mod inspect;
mod intervene;
mod library;
mod monitor;
mod output;
mod pipeline;
mod prep;
mod presets;
mod profile;
mod report;
mod runs;
mod scenarios;
mod serve;
mod simplify;
mod simulate;
mod templates;
mod validate;
mod workspace;
mod worldops;

pub use args::{parse_assignment_text, parse_identifier_list};
pub use config::CliConfig;
pub use discover::discovery_summary;
pub use error::CliError;
pub use inspect::world_summary;
pub use intervene::parse_scheduled_assignment as parse_scheduled_assignment_text;
pub use output::trajectory_csv;
pub use profile::profile_summary;
pub use serve::unavailable as serve_unavailable;
pub use simulate::simulation_config;

use std::{fmt::Write, fs, path::Path};

use lawsynth_bundle::{read_discrete_world, read_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, read_csv_numeric, read_parquet_numeric, read_tsv_numeric};
use lawsynth_differentiate::DerivativeMethod;
use lawsynth_discovery::{DiscoveryConfig, SparseMethod, discover};
use lawsynth_sim::{
    DiscreteSimulationConfig, SimulationConfig, SimulationRequest, simulate, simulate_discrete,
};
use lawsynth_stats::BootstrapConfig;

/// Runs a CLI invocation without process-global side effects.
pub fn run(arguments: &[String]) -> Result<String, String> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(usage());
    };
    match command {
        "inspect" if arguments.len() == 2 => inspect_command(&arguments[1]),
        "discover" => discover_command(&arguments[1..]),
        "simulate" => simulate_command(&arguments[1..]),
        "simulate-discrete" => simulate_discrete_command(&arguments[1..]),
        "report" => report::run(&arguments[1..]),
        "pipeline" => pipeline::run(&arguments[1..]),
        "explain" => explain::run(&arguments[1..]),
        "simplify" => simplify::run(&arguments[1..]),
        "compose" => compose::run(&arguments[1..]),
        "edit" => edit::run(&arguments[1..]),
        "compare" => compare::run(&arguments[1..]),
        "forecast" => forecast::run(&arguments[1..]),
        "prep" => prep::run(&arguments[1..]),
        "monitor" => monitor::run(&arguments[1..]),
        "scenarios" => scenarios::run(&arguments[1..]),
        "doctor" => doctor::run(&arguments[1..]),
        "library" => library::run(&arguments[1..]),
        "runs" => runs::run(&arguments[1..]),
        "profile" => profile::run(&arguments[1..]),
        "presets" => presets::run(&arguments[1..]),
        "export" => export::run(&arguments[1..]),
        "new" => templates::run_new(&arguments[1..]),
        "templates" => templates::run_templates(&arguments[1..]),
        "validate" => validate::run(&arguments[1..]),
        "backtest" => backtest::run(&arguments[1..]),
        "workspace" => workspace::run(&arguments[1..]),
        "help" | "--help" | "-h" => Ok(usage()),
        _ => Err(usage()),
    }
}

fn discover_command(arguments: &[String]) -> Result<String, String> {
    // A `--preset <name>` seeds the discovery defaults for this run; explicit
    // flags below override those seeds because they are parsed afterwards.
    // Capture the preset name before `extract` consumes the flag, so run
    // tracking can record which preset seeded the configuration.
    let preset_name = preset_name_of(arguments);
    let (preset, arguments) = presets::extract(arguments)?;
    let seed = preset.unwrap_or_default();
    let Some(input) = arguments.first() else {
        return Err(usage());
    };
    let mut time = None;
    let mut state = None;
    let mut output = None;
    let mut degree = seed.degree;
    let mut threshold = seed.threshold;
    let mut include_trigonometric = seed.include_trigonometric;
    let mut include_rational = seed.include_rational;
    let mut use_spline = false;
    let mut use_spectral = false;
    let mut savgol_window = None;
    let mut tvreg_lambda = None;
    let mut tvreg_iterations = 100;
    let mut smoothing_radius = None;
    let mut bootstrap_replicates = None;
    let mut symbolic_depth = None;
    let mut sparse_method = seed.sparse_method;
    let mut enable_regimes = false;
    let mut report_pareto = false;
    let mut enable_refine = seed.enable_refine;
    let mut enable_causal = false;
    let mut track = false;
    let mut label = None;
    let mut runs_dir = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = &arguments[index];
        if option == "--trigonometric"
            || option == "--rational"
            || option == "--spline"
            || option == "--spectral"
            || option == "--regimes"
            || option == "--pareto"
            || option == "--refine"
            || option == "--causal"
            || option == "--track"
        {
            match option.as_str() {
                "--trigonometric" => include_trigonometric = true,
                "--rational" => include_rational = true,
                "--spectral" => use_spectral = true,
                "--spline" => use_spline = true,
                "--regimes" => enable_regimes = true,
                "--pareto" => report_pareto = true,
                "--refine" => enable_refine = true,
                "--causal" => enable_causal = true,
                "--track" => track = true,
                _ => unreachable!(),
            }
            index += 1;
            continue;
        }
        let value = arguments.get(index + 1).ok_or_else(usage)?;
        match option.as_str() {
            "--time" => time = Some(value.clone()),
            "--label" => label = Some(value.clone()),
            "--runs-dir" => runs_dir = Some(value.clone()),
            "--state" => state = Some(parse_identifiers(value)?),
            "--output" => output = Some(value.clone()),
            "--degree" => degree = parse_steps(value)?,
            "--threshold" => threshold = parse_number(value)?,
            "--savgol-window" => savgol_window = Some(parse_steps(value)?),
            "--tvreg-lambda" => tvreg_lambda = Some(parse_number(value)?),
            "--tvreg-iterations" => tvreg_iterations = parse_steps(value)?,
            "--smooth-radius" => smoothing_radius = Some(parse_steps(value)?),
            "--bootstrap" => bootstrap_replicates = Some(parse_steps(value)?),
            "--symbolic-depth" => symbolic_depth = Some(parse_steps(value)?),
            "--solver" => {
                sparse_method = match value.as_str() {
                    "stlsq" => SparseMethod::Stlsq,
                    "sr3" => SparseMethod::Sr3,
                    _ => return Err("solver must be 'stlsq' or 'sr3'".to_owned()),
                }
            }
            _ => return Err(usage()),
        }
        index += 2;
    }
    let dataset = read_numeric_dataset(input, time.as_deref().ok_or_else(usage)?)?;
    let mut config = DiscoveryConfig::new(state.ok_or_else(usage)?);
    config.polynomial_degree = degree;
    config.sparse.threshold = threshold;
    config.sparse_method = sparse_method;
    config.include_trigonometric = include_trigonometric;
    config.include_rational = include_rational;
    config.smoothing_radius = smoothing_radius;
    if [use_spline, use_spectral, savgol_window.is_some(), tvreg_lambda.is_some()]
        .into_iter()
        .filter(|selected| *selected)
        .count()
        > 1
    {
        return Err("choose only one of --spline, --spectral, --savgol-window, or --tvreg-lambda"
            .to_owned());
    }
    if use_spline {
        config.derivative.method = DerivativeMethod::NaturalCubicSpline;
    } else if use_spectral {
        config.derivative.method = DerivativeMethod::Spectral;
    } else if let Some(window) = savgol_window {
        config.derivative.method = DerivativeMethod::SavitzkyGolay { window };
    } else if let Some(lambda) = tvreg_lambda {
        config.derivative.method =
            DerivativeMethod::TotalVariation { lambda, iterations: tvreg_iterations };
    }
    if let Some(replicates) = bootstrap_replicates {
        config.bootstrap = Some(BootstrapConfig { replicates, ..BootstrapConfig::default() });
    }
    if let Some(max_depth) = symbolic_depth {
        config.symbolic =
            Some(lawsynth_symbolic::SymbolicConfig { max_depth, ..Default::default() });
    }
    if enable_regimes {
        config.enable_regimes();
    }
    if enable_refine {
        config.enable_refinement();
    }
    if enable_causal {
        config.enable_causal_hypothesis();
    }
    let result = discover(&dataset, &config).map_err(|error| error.to_string())?;
    let frontier_size = result.frontier.len();
    let regime_segments = result.regimes.as_ref().map(|segmentation| segmentation.segments.len());
    let dependency_edges = result.dependency_hypothesis.as_ref().map(|graph| graph.edges().count());
    let candidate = result
        .candidates
        .into_iter()
        .next()
        .ok_or_else(|| "discovery produced no candidates".to_owned())?;
    write_world(output.ok_or_else(usage)?, &candidate.world).map_err(|error| error.to_string())?;
    let mut summary = format!(
        "discovered world: mse={:.6e}, complexity={}\n",
        candidate.metrics.mean_squared_error, candidate.metrics.complexity
    );
    if report_pareto {
        writeln!(&mut summary, "pareto frontier: {frontier_size} candidate(s)").unwrap();
    }
    if let Some(segments) = regime_segments {
        writeln!(&mut summary, "regimes: {segments} segment(s)").unwrap();
    }
    if let Some(refinement) = &candidate.refinement {
        writeln!(
            &mut summary,
            "refinement: improvement={:.6e}, iterations={}",
            refinement.improvement(),
            refinement.iterations
        )
        .unwrap();
    }
    if let Some(edges) = dependency_edges {
        writeln!(&mut summary, "dependency hypothesis: {edges} edge(s)").unwrap();
    }
    if track {
        // Record this run deterministically. The data hash is a content anchor
        // (no clock is read); the run's id is derived from it plus the config.
        let time_column = time.as_deref().unwrap_or("time");
        let derivative = if use_spline {
            "spline".to_owned()
        } else if use_spectral {
            "spectral".to_owned()
        } else if let Some(window) = savgol_window {
            format!("savgol:{window}")
        } else if let Some(lambda) = tvreg_lambda {
            format!("tvreg:{lambda:.6e}@{tvreg_iterations}")
        } else {
            "finite-difference".to_owned()
        };
        let solver = match sparse_method {
            SparseMethod::Stlsq => "stlsq",
            SparseMethod::Sr3 => "sr3",
        };
        let mut columns = vec![time_column.to_owned()];
        columns.extend(dataset.columns().keys().map(|id| id.as_str().to_owned()));
        let data_bytes = fs::read(input).map_err(|error| error.to_string())?;
        let record = runs::RunBuilder::new()
            .field("label", label.as_deref().unwrap_or("-"))
            .field("data.hash", lawsynth_bundle::sha256_hex(&data_bytes))
            .field("data.columns", columns.join(","))
            .field("data.samples", dataset.time().len().to_string())
            .field("config.preset", preset_name.as_str())
            .field("config.degree", degree.to_string())
            .field("config.threshold", format!("{threshold:.6e}"))
            .field("config.solver", solver)
            .field("config.derivative", derivative)
            .field(
                "config.smoothing",
                smoothing_radius.map(|radius| radius.to_string()).unwrap_or_else(|| "-".to_owned()),
            )
            .toggle("config.trigonometric", include_trigonometric)
            .toggle("config.rational", include_rational)
            .toggle("config.regimes", enable_regimes)
            .toggle("config.refine", enable_refine)
            .toggle("config.causal", enable_causal)
            .toggle("config.pareto", report_pareto)
            .field("result.mse", format!("{:.6e}", candidate.metrics.mean_squared_error))
            .field("result.complexity", candidate.metrics.complexity.to_string())
            .field("result.laws", config.state.len().to_string())
            .field("result.pareto_size", frontier_size.to_string())
            .field(
                "result.regime_segments",
                regime_segments.map(|count| count.to_string()).unwrap_or_else(|| "-".to_owned()),
            )
            .build();
        let message = runs::record_run(runs_dir.as_deref(), &record)?;
        summary.push_str(&message);
    }
    Ok(summary)
}

/// Extracts the `--preset NAME` value from raw arguments, or `"none"` if absent.
///
/// Read before `presets::extract` consumes the flag so run tracking can record
/// which preset seeded a configuration.
fn preset_name_of(arguments: &[String]) -> String {
    arguments
        .iter()
        .position(|argument| argument == "--preset")
        .and_then(|index| arguments.get(index + 1))
        .cloned()
        .unwrap_or_else(|| "none".to_owned())
}

/// Reads observations through the native CSV, TSV, or Parquet data boundary.
///
/// Binary Parquet is selected by extension before any text decoding. The native
/// Parquet decoder deliberately rejects encodings outside its supported numeric
/// subset instead of silently producing a partial dataset.
pub fn read_numeric_dataset(path: impl AsRef<Path>, time_column: &str) -> Result<Dataset, String> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let extension =
        path.extension().and_then(|extension| extension.to_str()).map(str::to_ascii_lowercase);
    let result = match extension.as_deref() {
        Some("parquet" | "parq") => read_parquet_numeric(&bytes, time_column),
        Some("tsv") => read_tsv_numeric(&bytes, time_column),
        Some("csv") | None => read_csv_numeric(&bytes, time_column),
        Some(extension) => {
            return Err(format!(
                "unsupported observation format '.{extension}'; use .csv, .tsv, .parquet, or .parq"
            ));
        }
    };
    result.map_err(|error| error.to_string())
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

fn inspect_command(bundle: &str) -> Result<String, String> {
    match read_world(bundle) {
        Ok(world) => Ok(format!(
            "continuous world: {} states, {} variables, {} parameters\n",
            world.state_ids().count(),
            world.variables().len(),
            world.parameters().len()
        )),
        Err(continuous_error) => match read_discrete_world(bundle) {
            Ok(world) => Ok(format!(
                "discrete world: {} states, {} variables, {} parameters\n",
                world.state_ids().count(),
                world.variables().len(),
                world.parameters().len()
            )),
            Err(_) => Err(continuous_error.to_string()),
        },
    }
}

fn simulate_command(arguments: &[String]) -> Result<String, String> {
    let Some(bundle) = arguments.first() else {
        return Err(usage());
    };
    let mut initial = Vec::new();
    let mut parameters = Vec::new();
    let mut inputs = Vec::new();
    let mut scheduled_parameters = Vec::new();
    let mut scheduled_inputs = Vec::new();
    let mut start = None;
    let mut end = None;
    let mut step = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = &arguments[index];
        let value = arguments.get(index + 1).ok_or_else(usage)?;
        match option.as_str() {
            "--initial" => initial.push(parse_assignment(value)?),
            "--parameter" => parameters.push(parse_assignment(value)?),
            "--input" => inputs.push(parse_assignment(value)?),
            "--parameter-at" => scheduled_parameters.push(parse_scheduled_assignment(value)?),
            "--input-at" => scheduled_inputs.push(parse_scheduled_assignment(value)?),
            "--start" => start = Some(parse_number(value)?),
            "--end" => end = Some(parse_number(value)?),
            "--step" => step = Some(parse_number(value)?),
            _ => return Err(usage()),
        }
        index += 2;
    }
    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let mut request = SimulationRequest::default();
    for (id, value) in initial {
        request = request.with_initial(id, value);
    }
    for (id, value) in parameters {
        request = request.with_parameter_override(id, value);
    }
    for (id, value) in inputs {
        request = request.with_input(id, value);
    }
    for (time, id, value) in scheduled_parameters {
        request = request.with_scheduled_parameter(time, id, value);
    }
    for (time, id, value) in scheduled_inputs {
        request = request.with_scheduled_input(time, id, value);
    }
    let trajectory = simulate(
        &world,
        SimulationConfig::new(
            start.ok_or_else(usage)?,
            end.ok_or_else(usage)?,
            step.ok_or_else(usage)?,
        )
        .map_err(|error| error.to_string())?,
        &request,
    )
    .map_err(|error| error.to_string())?;
    let mut output = String::from("time");
    for state in world.state_ids() {
        write!(&mut output, ",{state}").unwrap();
    }
    output.push('\n');
    for row in 0..trajectory.samples() {
        write!(&mut output, "{:.17e}", trajectory.time[row]).unwrap();
        for state in world.state_ids() {
            write!(&mut output, ",{:.17e}", trajectory.values[state][row]).unwrap();
        }
        output.push('\n');
    }
    Ok(output)
}

fn simulate_discrete_command(arguments: &[String]) -> Result<String, String> {
    let Some(bundle) = arguments.first() else {
        return Err(usage());
    };
    let mut initial = Vec::new();
    let mut parameters = Vec::new();
    let mut inputs = Vec::new();
    let mut scheduled_parameters = Vec::new();
    let mut scheduled_inputs = Vec::new();
    let mut start = 0.0;
    let mut steps = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = &arguments[index];
        let value = arguments.get(index + 1).ok_or_else(usage)?;
        match option.as_str() {
            "--initial" => initial.push(parse_assignment(value)?),
            "--parameter" => parameters.push(parse_assignment(value)?),
            "--input" => inputs.push(parse_assignment(value)?),
            "--parameter-at" => scheduled_parameters.push(parse_scheduled_assignment(value)?),
            "--input-at" => scheduled_inputs.push(parse_scheduled_assignment(value)?),
            "--start" => start = parse_number(value)?,
            "--steps" => steps = Some(parse_steps(value)?),
            _ => return Err(usage()),
        }
        index += 2;
    }
    let world = read_discrete_world(bundle).map_err(|error| error.to_string())?;
    let mut request = SimulationRequest::default();
    for (id, value) in initial {
        request = request.with_initial(id, value);
    }
    for (id, value) in parameters {
        request = request.with_parameter_override(id, value);
    }
    for (id, value) in inputs {
        request = request.with_input(id, value);
    }
    for (time, id, value) in scheduled_parameters {
        request = request.with_scheduled_parameter(time, id, value);
    }
    for (time, id, value) in scheduled_inputs {
        request = request.with_scheduled_input(time, id, value);
    }
    let trajectory = simulate_discrete(
        &world,
        DiscreteSimulationConfig::new(start, steps.ok_or_else(usage)?)
            .map_err(|error| error.to_string())?,
        &request,
    )
    .map_err(|error| error.to_string())?;
    let mut output = String::from("time");
    for state in world.state_ids() {
        write!(&mut output, ",{state}").unwrap();
    }
    output.push('\n');
    for row in 0..trajectory.samples() {
        write!(&mut output, "{:.17e}", trajectory.time[row]).unwrap();
        for state in world.state_ids() {
            write!(&mut output, ",{:.17e}", trajectory.values[state][row]).unwrap();
        }
        output.push('\n');
    }
    Ok(output)
}

fn parse_assignment(value: &str) -> Result<(Identifier, f64), String> {
    let (name, number) = value.split_once('=').ok_or_else(|| "expected NAME=VALUE".to_owned())?;
    Ok((Identifier::new(name).map_err(|error| error.to_string())?, parse_number(number)?))
}

fn parse_scheduled_assignment(value: &str) -> Result<(f64, Identifier, f64), String> {
    let (time, assignment) =
        value.split_once(':').ok_or_else(|| "expected TIME:NAME=VALUE".to_owned())?;
    let (id, value) = parse_assignment(assignment)?;
    Ok((parse_number(time)?, id, value))
}

fn parse_number(value: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}'"))?;
    if number.is_finite() { Ok(number) } else { Err(format!("number '{value}' must be finite")) }
}

fn parse_steps(value: &str) -> Result<usize, String> {
    value.parse().map_err(|_| format!("invalid step count '{value}'"))
}

fn usage() -> String {
    "usage:\n  lawsynth inspect WORLD.lsworld\n  lawsynth discover OBSERVATIONS.{csv,tsv,parquet} --time COLUMN --state NAME[,NAME...] --output WORLD.lsworld [--preset NAME] [--degree N] [--threshold VALUE] [--solver stlsq|sr3] [--trigonometric] [--rational] [--savgol-window ODD_N | --spline | --spectral | --tvreg-lambda VALUE [--tvreg-iterations N]] [--smooth-radius N] [--bootstrap REPLICATES] [--symbolic-depth N] [--regimes] [--pareto] [--refine] [--causal] [--track [--label TEXT] [--runs-dir DIR]]\n  lawsynth prep OBSERVATIONS.{csv,tsv,parquet} [--time COLUMN] --output CLEAN.csv [--trim START:END] [--drop-constant] [--detrend] [--smooth-window N] [--resample DT]\n  lawsynth monitor WORLD.lsworld --data NEW.{csv,tsv,parquet} [--time COLUMN] [--threshold K]\n  lawsynth profile OBSERVATIONS.{csv,tsv,parquet} [--time COLUMN] [--json]\n  lawsynth runs <list|show|compare> [--dir DIR] ...\n  lawsynth simulate WORLD.lsworld --initial NAME=VALUE [--initial NAME=VALUE] --start T --end T --step DT [--parameter NAME=VALUE] [--input NAME=VALUE] [--parameter-at TIME:NAME=VALUE] [--input-at TIME:NAME=VALUE]\n  lawsynth simulate-discrete WORLD.lsworld --initial NAME=VALUE [--initial NAME=VALUE] --steps N [--start T] [--parameter NAME=VALUE] [--input NAME=VALUE] [--parameter-at TIME:NAME=VALUE] [--input-at TIME:NAME=VALUE]\n  lawsynth report WORLD.lsworld [--output REPORT.html] [--title TEXT] [--start T] [--end T] [--step DT] [--initial NAME=VALUE]... [--data OBS.{csv,tsv,parquet}] [--time COLUMN]\n  lawsynth pipeline PIPELINE.toml | lawsynth pipeline --example\n  lawsynth explain WORLD.lsworld\n  lawsynth simplify WORLD.lsworld [--output SIMPLIFIED.lsworld]\n  lawsynth compose WORLD-A.lsworld WORLD-B.lsworld --output COMBINED.lsworld [--prefix-a A_] [--prefix-b B_]\n  lawsynth edit WORLD.lsworld --output EDITED.lsworld [--rename OLD:NEW] [--set-param NAME=VALUE] [--drop-law TARGET] [--scale-law TARGET=FACTOR]\n  lawsynth compare WORLD-A.lsworld WORLD-B.lsworld [--json] [--html FILE]\n  lawsynth forecast WORLD.lsworld [--horizon T] [--start T] [--step DT] [--initial NAME=VALUE]... [--parameter NAME=VALUE]... [--intervene NAME=VALUE@TIME]... [--output FORECAST.csv] [--confidence --data OBS.{csv,tsv,parquet} [--time COLUMN] [--level L] [--replicates N] [--seed N] [--html BANDS.html]]\n  lawsynth scenarios WORLD.lsworld [--horizon T] [--start T] [--step DT] [--initial NAME=VALUE]... --scenario NAME[:k=v@t,...] [--scenario ...] [--html FILE]\n  lawsynth doctor\n  lawsynth library <add|list|show|search|compare|remove> [--dir DIR] ...\n  lawsynth presets\n  lawsynth templates\n  lawsynth new TEMPLATE [--output WORLD.lsworld] [--data OBS.csv] [--samples N]\n  lawsynth export WORLD.lsworld --format <python|c|onnx|matlab|latex|json> [--output FILE]\n  lawsynth validate WORLD.lsworld --data OBS.{csv,tsv,parquet} [--time COLUMN] [--holdout FRACTION]\n  lawsynth backtest WORLD.lsworld --data OBS.{csv,tsv,parquet} [--time COLUMN] [--origins N] [--horizon H] [--html REPORT.html]\n  lawsynth workspace <export|import> ARCHIVE.lsworkspace [--dir DIR] [--force]\n\nRun any command with --help for details.".to_owned()
}
