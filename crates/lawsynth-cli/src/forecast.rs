//! `lawsynth forecast` — simulate a world forward with what-if interventions.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_report::format_number;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::{VariableRole, World};

use crate::args::parse_assignment_text;
use crate::output::trajectory_csv;

const DEFAULT_HORIZON: f64 = 20.0;
const DEFAULT_START: f64 = 0.0;
const DEFAULT_STEP: f64 = 0.1;
const DEFAULT_INITIAL: f64 = 1.0;

/// Help text for `lawsynth forecast`.
pub fn help() -> String {
    "lawsynth forecast WORLD.lsworld [--horizon T] [--start T] [--step DT] \
[--initial NAME=VALUE]... [--parameter NAME=VALUE]... [--intervene NAME=VALUE@TIME]... \
[--output FORECAST.csv]\n\n\
Simulates the world forward to a forecast horizon, applying constant parameter \
overrides and scheduled what-if interventions, and emits the trajectory as CSV \
plus a summary."
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
}

/// Runs the `forecast` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let world = read_world(&args.bundle).map_err(|error| error.to_string())?;
    let request = build_request(&world, &args)?;
    let config = SimulationConfig::new(args.start, args.horizon, args.step)
        .map_err(|error| error.to_string())?;
    let trajectory = simulate(&world, config, &request).map_err(|error| error.to_string())?;
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
    };
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
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
}
