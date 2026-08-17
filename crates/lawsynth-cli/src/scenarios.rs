//! `lawsynth scenarios` — define and compare multiple named what-if scenarios.
//!
//! Each scenario is a set of scheduled parameter/input interventions layered on
//! top of a shared baseline simulation. The command simulates the baseline plus
//! every scenario over one horizon (reusing the same scheduled-intervention sim
//! path as `simulate` and `forecast`), then presents them together: a stdout
//! comparison table (final state per variable and its divergence from baseline,
//! plus the interventions that define each scenario) and, with `--html`, a
//! self-contained report with all trajectories overlaid on one multi-series
//! chart per state. This is the "compare your options and decide" surface.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_report::{ScenarioOutcome, ScenarioReport, format_number, render_scenarios};
use lawsynth_sim::{SimulationConfig, SimulationRequest, Trajectory, simulate};
use lawsynth_world::{VariableRole, World};

use crate::args::parse_assignment_text;

const DEFAULT_HORIZON: f64 = 20.0;
const DEFAULT_START: f64 = 0.0;
const DEFAULT_STEP: f64 = 0.1;
const DEFAULT_INITIAL: f64 = 1.0;
const BASELINE_LABEL: &str = "baseline";

/// Help text for `lawsynth scenarios`.
pub fn help() -> String {
    "lawsynth scenarios WORLD.lsworld [--horizon T] [--start T] [--step DT] \
[--initial NAME=VALUE]... --scenario NAME[:k=v@t,...] [--scenario ...] [--html FILE]\n\n\
Defines and simulates multiple named what-if scenarios (each a set of scheduled \
NAME=VALUE@TIME parameter/input interventions) plus an implicit baseline, then \
compares them. stdout shows a table of the final state per variable and its \
divergence from baseline, with the interventions that define each scenario. With \
--html, writes a self-contained report overlaying every scenario's trajectory on \
one multi-series chart per state. Deterministic and offline."
        .to_owned()
}

/// A single named scenario parsed from a `--scenario` spec.
struct Scenario {
    label: String,
    /// Human-readable, normalized description of the interventions.
    description: String,
    /// Scheduled interventions as `(time, target, value)`.
    interventions: Vec<(f64, Identifier, f64)>,
}

struct ScenarioArgs {
    bundle: String,
    horizon: f64,
    start: f64,
    step: f64,
    initials: Vec<(Identifier, f64)>,
    scenarios: Vec<Scenario>,
    html: Option<String>,
}

/// Runs the `scenarios` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let world = read_world(&args.bundle).map_err(|error| error.to_string())?;
    validate_initials(&world, &args.initials)?;

    let states: Vec<String> = world.state_ids().map(|id| id.as_str().to_owned()).collect();
    let config = SimulationConfig::new(args.start, args.horizon, args.step)
        .map_err(|error| error.to_string())?;

    // Baseline first, then every named scenario, all sharing the same horizon.
    let mut outcomes: Vec<ScenarioOutcome> = Vec::with_capacity(args.scenarios.len() + 1);
    let baseline_trajectory = simulate_scenario(&world, config, &args.initials, &[])?;
    let time = baseline_trajectory.time.clone();
    outcomes.push(to_outcome(BASELINE_LABEL, "(baseline)", true, &baseline_trajectory));

    for scenario in &args.scenarios {
        let trajectory = simulate_scenario(&world, config, &args.initials, &scenario.interventions)
            .map_err(|error| format!("scenario '{}': {error}", scenario.label))?;
        outcomes.push(to_outcome(&scenario.label, &scenario.description, false, &trajectory));
    }

    let report = ScenarioReport {
        title: format!("LawSynth scenarios: {}", args.bundle),
        time,
        states: states.clone(),
        scenarios: outcomes,
    };

    let mut summary = render_table(&args.bundle, &report);
    if let Some(path) = &args.html {
        let document = render_scenarios(&report);
        fs::write(path, &document).map_err(|error| format!("failed to write {path}: {error}"))?;
        let _ = writeln!(summary, "\nwrote scenario report: {path} ({} bytes)", document.len());
    }
    Ok(summary)
}

/// Simulates one scenario: shared initial conditions plus its interventions.
fn simulate_scenario(
    world: &World,
    config: SimulationConfig,
    initials: &[(Identifier, f64)],
    interventions: &[(f64, Identifier, f64)],
) -> Result<Trajectory, String> {
    let overrides: BTreeMap<Identifier, f64> = initials.iter().cloned().collect();
    let mut request = SimulationRequest::default();
    for state in world.state_ids() {
        let value = overrides.get(state).copied().unwrap_or(DEFAULT_INITIAL);
        request = request.with_initial(state.clone(), value);
    }
    for (time, id, value) in interventions {
        // Route each scheduled change to a parameter or a non-state input,
        // matching the existing simulate / forecast intervention path.
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
    simulate(world, config, &request).map_err(|error| error.to_string())
}

fn to_outcome(
    label: &str,
    description: &str,
    is_baseline: bool,
    trajectory: &Trajectory,
) -> ScenarioOutcome {
    let trajectories = trajectory
        .values
        .iter()
        .map(|(id, values)| (id.as_str().to_owned(), values.clone()))
        .collect();
    ScenarioOutcome {
        label: label.to_owned(),
        interventions: description.to_owned(),
        is_baseline,
        trajectories,
    }
}

/// Renders the stdout comparison table: final state and divergence per scenario.
fn render_table(bundle: &str, report: &ScenarioReport) -> String {
    let named = report.scenarios.iter().filter(|outcome| !outcome.is_baseline).count();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "Scenarios over t in [{}, {}], step {} ({} samples); world {bundle}",
        format_number(report.time.first().copied().unwrap_or(0.0)),
        format_number(report.time.last().copied().unwrap_or(0.0)),
        format_number(step_of(&report.time)),
        report.time.len(),
    );
    let _ = writeln!(
        out,
        "{named} scenario(s) + baseline, {} state variable(s)\n",
        report.states.len()
    );

    // Column layout: Scenario | Interventions | (state, Δstate)*
    let mut header: Vec<String> = vec!["scenario".to_owned(), "interventions".to_owned()];
    for state in &report.states {
        header.push(state.clone());
        header.push(format!("d{state}"));
    }

    let baseline_finals: BTreeMap<&str, f64> =
        report.states.iter().map(|state| (state.as_str(), final_value(report, 0, state))).collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    for (index, outcome) in report.scenarios.iter().enumerate() {
        let mut row = vec![outcome.label.clone(), outcome.interventions.clone()];
        for state in &report.states {
            let final_value = final_value(report, index, state);
            row.push(format_number(final_value));
            if outcome.is_baseline {
                row.push("-".to_owned());
            } else {
                let base = baseline_finals.get(state.as_str()).copied().unwrap_or(f64::NAN);
                row.push(signed(final_value - base));
            }
        }
        rows.push(row);
    }

    let widths = column_widths(&header, &rows);
    write_row(&mut out, &header, &widths);
    for row in &rows {
        write_row(&mut out, row, &widths);
    }
    out
}

fn final_value(report: &ScenarioReport, scenario_index: usize, state: &str) -> f64 {
    report.scenarios[scenario_index]
        .trajectories
        .get(state)
        .and_then(|values| values.iter().rev().find(|value| value.is_finite()).copied())
        .unwrap_or(f64::NAN)
}

fn step_of(time: &[f64]) -> f64 {
    match time {
        [first, second, ..] => second - first,
        _ => 0.0,
    }
}

fn column_widths(header: &[String], rows: &[Vec<String>]) -> Vec<usize> {
    let mut widths: Vec<usize> = header.iter().map(String::len).collect();
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            if index < widths.len() {
                widths[index] = widths[index].max(cell.len());
            }
        }
    }
    widths
}

fn write_row(out: &mut String, row: &[String], widths: &[usize]) {
    for (index, cell) in row.iter().enumerate() {
        if index > 0 {
            out.push_str("  ");
        }
        let width = widths.get(index).copied().unwrap_or(0);
        let _ = write!(out, "{cell:<width$}");
    }
    out.push('\n');
}

/// Formats a divergence with an explicit sign.
fn signed(delta: f64) -> String {
    if delta == 0.0 || !delta.is_finite() {
        return format_number(delta);
    }
    let magnitude = format_number(delta.abs());
    if delta > 0.0 { format!("+{magnitude}") } else { format!("-{magnitude}") }
}

fn validate_initials(world: &World, initials: &[(Identifier, f64)]) -> Result<(), String> {
    let states: BTreeSet<&Identifier> = world.state_ids().collect();
    for (id, _) in initials {
        if !states.contains(id) {
            return Err(format!("--initial target '{}' is not a state variable", id.as_str()));
        }
    }
    Ok(())
}

fn parse(arguments: &[String]) -> Result<ScenarioArgs, String> {
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle == "--help" || bundle == "-h" {
        return Err(help());
    }
    let mut args = ScenarioArgs {
        bundle: bundle.clone(),
        horizon: DEFAULT_HORIZON,
        start: DEFAULT_START,
        step: DEFAULT_STEP,
        initials: Vec::new(),
        scenarios: Vec::new(),
        html: None,
    };
    let mut seen: BTreeSet<String> = BTreeSet::new();
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
            "--scenario" => {
                let scenario = parse_scenario(value)?;
                if !seen.insert(scenario.label.clone()) {
                    return Err(format!("duplicate scenario name '{}'", scenario.label));
                }
                args.scenarios.push(scenario);
            }
            "--html" => args.html = Some(value.clone()),
            _ => return Err(help()),
        }
        index += 2;
    }
    if args.scenarios.is_empty() {
        return Err("at least one --scenario is required".to_owned());
    }
    Ok(args)
}

/// Parses a `NAME[:k=v@t,k2=v2@t2,...]` scenario spec.
fn parse_scenario(spec: &str) -> Result<Scenario, String> {
    let (name, rest) = match spec.split_once(':') {
        Some((name, rest)) => (name, rest),
        None => (spec, ""),
    };
    let name = name.trim();
    if name.is_empty() {
        return Err(format!("scenario spec '{spec}' has an empty name"));
    }
    if name.eq_ignore_ascii_case(BASELINE_LABEL) {
        return Err("'baseline' is reserved; choose another scenario name".to_owned());
    }
    let mut interventions = Vec::new();
    for piece in rest.split(',').map(str::trim).filter(|piece| !piece.is_empty()) {
        interventions.push(parse_intervention(piece)?);
    }
    let description = if interventions.is_empty() {
        "(no interventions)".to_owned()
    } else {
        interventions
            .iter()
            .map(|(time, id, value)| {
                format!("{}={}@{}", id.as_str(), format_number(*value), format_number(*time))
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    Ok(Scenario { label: name.to_owned(), description, interventions })
}

/// Parses the `NAME=VALUE@TIME` scheduled-intervention syntax.
fn parse_intervention(value: &str) -> Result<(f64, Identifier, f64), String> {
    let (assignment, time) =
        value.split_once('@').ok_or_else(|| format!("expected NAME=VALUE@TIME in '{value}'"))?;
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
    fn parses_scenario_with_interventions() {
        let scenario = parse_scenario("shock:k=2.0@5,rho=1.5@8").unwrap();
        assert_eq!(scenario.label, "shock");
        assert_eq!(scenario.interventions.len(), 2);
        assert_eq!(scenario.interventions[0], (5.0, id("k"), 2.0));
        assert!(scenario.description.contains("k=2@5"));
    }

    #[test]
    fn parses_scenario_without_interventions() {
        let scenario = parse_scenario("hold").unwrap();
        assert!(scenario.interventions.is_empty());
        assert_eq!(scenario.description, "(no interventions)");
    }

    #[test]
    fn rejects_reserved_baseline_name() {
        assert!(parse_scenario("baseline:k=1@2").is_err());
    }

    #[test]
    fn rejects_malformed_intervention() {
        assert!(parse_scenario("bad:k=2").is_err());
    }

    #[test]
    fn signed_divergence_has_explicit_sign() {
        assert_eq!(signed(0.15), "+0.15");
        assert_eq!(signed(-0.15), "-0.15");
        assert_eq!(signed(0.0), "0");
    }
}
