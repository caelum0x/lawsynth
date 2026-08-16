//! Command implementation shared by the LawSynth binary and integration tests.

mod args;
mod config;
mod discover;
mod error;
mod inspect;
mod intervene;
mod output;
mod profile;
mod serve;
mod simulate;

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

use std::{fmt::Write, fs};

use lawsynth_bundle::{read_discrete_world, read_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
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
        _ => Err(usage()),
    }
}

fn discover_command(arguments: &[String]) -> Result<String, String> {
    let Some(input) = arguments.first() else {
        return Err(usage());
    };
    let mut time = None;
    let mut state = None;
    let mut output = None;
    let mut degree = 2;
    let mut threshold = 0.05;
    let mut include_trigonometric = false;
    let mut include_rational = false;
    let mut use_spline = false;
    let mut use_spectral = false;
    let mut savgol_window = None;
    let mut tvreg_lambda = None;
    let mut tvreg_iterations = 100;
    let mut smoothing_radius = None;
    let mut bootstrap_replicates = None;
    let mut symbolic_depth = None;
    let mut sparse_method = SparseMethod::Stlsq;
    let mut index = 1;
    while index < arguments.len() {
        let option = &arguments[index];
        if option == "--trigonometric"
            || option == "--rational"
            || option == "--spline"
            || option == "--spectral"
        {
            if option == "--trigonometric" {
                include_trigonometric = true;
            } else if option == "--rational" {
                include_rational = true;
            } else if option == "--spectral" {
                use_spectral = true;
            } else {
                use_spline = true;
            }
            index += 1;
            continue;
        }
        let value = arguments.get(index + 1).ok_or_else(usage)?;
        match option.as_str() {
            "--time" => time = Some(value.clone()),
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
    let dataset = read_numeric_csv(input, time.as_deref().ok_or_else(usage)?)?;
    let mut config = DiscoveryConfig::new(state.ok_or_else(usage)?);
    config.polynomial_degree = degree;
    config.sparse.threshold = threshold;
    config.sparse_method = sparse_method;
    config.include_trigonometric = include_trigonometric;
    config.include_rational = include_rational;
    config.smoothing_radius = smoothing_radius;
    if [
        use_spline,
        use_spectral,
        savgol_window.is_some(),
        tvreg_lambda.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count()
        > 1
    {
        return Err(
            "choose only one of --spline, --spectral, --savgol-window, or --tvreg-lambda"
                .to_owned(),
        );
    }
    if use_spline {
        config.derivative.method = DerivativeMethod::NaturalCubicSpline;
    } else if use_spectral {
        config.derivative.method = DerivativeMethod::Spectral;
    } else if let Some(window) = savgol_window {
        config.derivative.method = DerivativeMethod::SavitzkyGolay { window };
    } else if let Some(lambda) = tvreg_lambda {
        config.derivative.method = DerivativeMethod::TotalVariation {
            lambda,
            iterations: tvreg_iterations,
        };
    }
    if let Some(replicates) = bootstrap_replicates {
        config.bootstrap = Some(BootstrapConfig {
            replicates,
            ..BootstrapConfig::default()
        });
    }
    if let Some(max_depth) = symbolic_depth {
        config.symbolic = Some(lawsynth_symbolic::SymbolicConfig {
            max_depth,
            ..Default::default()
        });
    }
    let result = discover(&dataset, &config).map_err(|error| error.to_string())?;
    let candidate = result
        .candidates
        .into_iter()
        .next()
        .ok_or_else(|| "discovery produced no candidates".to_owned())?;
    write_world(output.ok_or_else(usage)?, &candidate.world).map_err(|error| error.to_string())?;
    Ok(format!(
        "discovered world: mse={:.6e}, complexity={}\n",
        candidate.metrics.mean_squared_error, candidate.metrics.complexity
    ))
}

fn read_numeric_csv(path: &str, time_column: &str) -> Result<Dataset, String> {
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut lines = contents.lines();
    let header = lines
        .next()
        .ok_or_else(|| "CSV input has no header".to_owned())?;
    let names = header.split(',').map(str::trim).collect::<Vec<_>>();
    if names.is_empty() || names.iter().any(|name| name.is_empty()) {
        return Err("CSV header has an empty column name".to_owned());
    }
    let time_index = names
        .iter()
        .position(|name| *name == time_column)
        .ok_or_else(|| format!("CSV has no '{time_column}' time column"))?;
    let ids = names
        .iter()
        .map(|name| Identifier::new(*name).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let mut values = vec![Vec::new(); names.len()];
    for (line_number, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
        if fields.len() != names.len() {
            return Err(format!(
                "CSV row {} has the wrong column count",
                line_number + 2
            ));
        }
        for (column, field) in fields.iter().enumerate() {
            values[column].push(parse_number(field)?);
        }
    }
    let time = TimeAxis::new(values[time_index].clone()).map_err(|error| error.to_string())?;
    let columns = ids
        .into_iter()
        .zip(values)
        .enumerate()
        .filter(|(index, _)| *index != time_index)
        .map(|(_, (id, values))| NumericColumn::new(id, values))
        .collect::<Vec<_>>();
    Dataset::new(time, columns).map_err(|error| error.to_string())
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
    let (name, number) = value
        .split_once('=')
        .ok_or_else(|| "expected NAME=VALUE".to_owned())?;
    Ok((
        Identifier::new(name).map_err(|error| error.to_string())?,
        parse_number(number)?,
    ))
}

fn parse_scheduled_assignment(value: &str) -> Result<(f64, Identifier, f64), String> {
    let (time, assignment) = value
        .split_once(':')
        .ok_or_else(|| "expected TIME:NAME=VALUE".to_owned())?;
    let (id, value) = parse_assignment(assignment)?;
    Ok((parse_number(time)?, id, value))
}

fn parse_number(value: &str) -> Result<f64, String> {
    let number: f64 = value
        .parse()
        .map_err(|_| format!("invalid number '{value}'"))?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(format!("number '{value}' must be finite"))
    }
}

fn parse_steps(value: &str) -> Result<usize, String> {
    value
        .parse()
        .map_err(|_| format!("invalid step count '{value}'"))
}

fn usage() -> String {
    "usage:\n  lawsynth inspect WORLD.lsworld\n  lawsynth discover OBSERVATIONS.csv --time COLUMN --state NAME[,NAME...] --output WORLD.lsworld [--degree N] [--threshold VALUE] [--solver stlsq|sr3] [--trigonometric] [--rational] [--savgol-window ODD_N | --spline | --spectral | --tvreg-lambda VALUE [--tvreg-iterations N]] [--smooth-radius N] [--bootstrap REPLICATES] [--symbolic-depth N]\n  lawsynth simulate WORLD.lsworld --initial NAME=VALUE [--initial NAME=VALUE] --start T --end T --step DT [--parameter NAME=VALUE] [--input NAME=VALUE] [--parameter-at TIME:NAME=VALUE] [--input-at TIME:NAME=VALUE]\n  lawsynth simulate-discrete WORLD.lsworld --initial NAME=VALUE [--initial NAME=VALUE] --steps N [--start T] [--parameter NAME=VALUE] [--input NAME=VALUE] [--parameter-at TIME:NAME=VALUE] [--input-at TIME:NAME=VALUE]".to_owned()
}
