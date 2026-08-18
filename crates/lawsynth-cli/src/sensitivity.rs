//! `lawsynth sensitivity` — forward (variational) sensitivity analysis.
//!
//! Integrates the forward-sensitivity equations of a discovered model
//! `ẋ = f(x; θ)` to obtain the trajectory sensitivities `∂x_i(t)/∂θ_j`: how each
//! forecast component responds to a change in a discovered coefficient. All of
//! the numerics come from [`lawsynth_sensitivity::forward_sensitivities`], which
//! reuses the analytic Jacobian and RK4 integrator.
//!
//! # What the world must provide
//!
//! Sensitivity is taken with respect to **parameter symbols** that appear in the
//! laws. Each name given to `--parameters` must be a declared world parameter
//! (its value is read from the world); those symbols are left free while every
//! *other* declared parameter is pinned at its stored value. A world that inlines
//! its coefficients as plain constants exposes no parameter to differentiate —
//! the command reports that plainly. A parameter that is declared but never used
//! differentiates to exactly zero, which is the correct, non-fabricated answer.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_report::format_number;
use lawsynth_sensitivity::{SensitivityConfig, SensitivityTrajectory, forward_sensitivities};
use lawsynth_world::World;

use crate::analysis::{
    fields_with_free, json_string, parse_assignment, parse_identifiers, parse_number,
    parse_positive, parse_usize,
};

/// Default integration step (mirrors the engine default).
const DEFAULT_DT: f64 = 1e-2;
/// Default number of integration steps (mirrors the engine default).
const DEFAULT_STEPS: usize = 100;

/// Help text for `lawsynth sensitivity`.
pub fn help() -> String {
    "lawsynth sensitivity WORLD.lsworld --parameters NAME[,NAME...] \
[--initial NAME=VALUE]... [--start T] [--dt DT] [--steps N] [--json]\n\n\
Integrates the forward-sensitivity equations of the world's field and reports \
the trajectory sensitivities dx_i/dtheta_j at the final time. Each --parameters \
name must be a declared world parameter (its value is read from the world); other \
declared parameters are held at their stored values. Initial state components \
default to 0 unless set with --initial. A parameter that never appears in the \
laws has exactly zero sensitivity. --json emits a stable machine-readable report."
        .to_owned()
}

/// Runs the `sensitivity` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle.starts_with('-') {
        return Err(help());
    }

    let mut parameters = None;
    let mut initial = Vec::new();
    let mut start = None;
    let mut dt = DEFAULT_DT;
    let mut steps = DEFAULT_STEPS;
    let mut as_json = false;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--json" {
            as_json = true;
            index += 1;
            continue;
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--parameters" => parameters = Some(parse_identifiers(value)?),
            "--initial" => initial.push(parse_assignment(value)?),
            "--start" => start = Some(parse_number(value, "--start")?),
            "--dt" => dt = parse_positive(value, "--dt")?,
            "--steps" => steps = parse_usize(value, "--steps")?,
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = lawsynth_bundle::read_world(bundle).map_err(|error| error.to_string())?;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    let parameters =
        parameters.ok_or_else(|| "--parameters NAME[,NAME...] is required".to_owned())?;

    let parameter_values = resolve_parameter_values(&world, &parameters)?;
    let initial_state = resolve_initial(&states, &initial)?;
    let fields = fields_with_free(&world, &parameters);

    let config = SensitivityConfig::new(start.unwrap_or(0.0), dt, steps);

    let trajectory = forward_sensitivities(
        &fields,
        &states,
        &parameters,
        &initial_state,
        &parameter_values,
        &config,
    )
    .map_err(|error| error.to_string())?;

    if as_json {
        Ok(render_json(bundle, &trajectory))
    } else {
        Ok(render_text(bundle, &initial_state, &parameter_values, &trajectory))
    }
}

/// Reads each requested parameter's value from the world's declared parameters.
fn resolve_parameter_values(world: &World, parameters: &[Identifier]) -> Result<Vec<f64>, String> {
    parameters
        .iter()
        .map(|name| {
            world.parameters().get(name).map(|parameter| parameter.value).ok_or_else(|| {
                format!(
                    "parameter '{}' is not a declared parameter of the world; \
--parameters names must be world parameters with a stored value",
                    name.as_str()
                )
            })
        })
        .collect()
}

/// Builds the initial-state vector in `states` order, defaulting unset states to
/// zero and rejecting an assignment to an unknown state.
fn resolve_initial(
    states: &[Identifier],
    assignments: &[(Identifier, f64)],
) -> Result<Vec<f64>, String> {
    for (name, _) in assignments {
        if !states.contains(name) {
            return Err(format!("--initial '{}' is not a state of the world", name.as_str()));
        }
    }
    Ok(states
        .iter()
        .map(|state| {
            assignments
                .iter()
                .find(|(name, _)| name == state)
                .map(|(_, value)| *value)
                .unwrap_or(0.0)
        })
        .collect())
}

/// Human-facing report: the sensitivity matrix at the final integration time.
fn render_text(
    bundle: &str,
    initial: &[f64],
    parameter_values: &[f64],
    trajectory: &SensitivityTrajectory,
) -> String {
    let mut out = String::new();
    let last = trajectory.sample_count() - 1;
    let final_time = trajectory.times()[last];

    let _ = writeln!(out, "Forward sensitivity analysis of {bundle}");
    let states: Vec<&str> = trajectory.states().iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states:     {}", states.join(", "));
    let parameters: Vec<String> = trajectory
        .parameters()
        .iter()
        .zip(parameter_values)
        .map(|(name, value)| format!("{}={}", name.as_str(), format_number(*value)))
        .collect();
    let _ = writeln!(out, "  parameters: {}", parameters.join(", "));
    let initial_line = crate::analysis::render_coordinates(trajectory.states(), initial);
    let _ = writeln!(out, "  initial:    {initial_line}");
    let _ = writeln!(
        out,
        "  integrated to t = {} over {} step(s)",
        format_number(final_time),
        trajectory.sample_count() - 1
    );
    out.push('\n');

    let _ = writeln!(out, "Sensitivities d x_i / d theta_j at t = {}:", format_number(final_time));
    for (state_index, state) in trajectory.states().iter().enumerate() {
        for (parameter_index, parameter) in trajectory.parameters().iter().enumerate() {
            let value = trajectory.partial(state_index, parameter_index, last).unwrap_or(0.0);
            let _ = writeln!(
                out,
                "  d {} / d {} = {}",
                state.as_str(),
                parameter.as_str(),
                format_number(value)
            );
        }
    }
    out
}

/// Stable, machine-readable report of the final-time sensitivity matrix.
fn render_json(bundle: &str, trajectory: &SensitivityTrajectory) -> String {
    let mut out = String::new();
    let last = trajectory.sample_count() - 1;
    let final_time = trajectory.times()[last];

    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let states: Vec<String> =
        trajectory.states().iter().map(|state| json_string(state.as_str())).collect();
    let parameters: Vec<String> =
        trajectory.parameters().iter().map(|name| json_string(name.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", states.join(", "));
    let _ = writeln!(out, "  \"parameters\": [{}],", parameters.join(", "));
    let _ = writeln!(out, "  \"final_time\": {final_time:.17e},");
    let _ = writeln!(out, "  \"sensitivities\": [");
    let mut entries = Vec::new();
    for (state_index, state) in trajectory.states().iter().enumerate() {
        for (parameter_index, parameter) in trajectory.parameters().iter().enumerate() {
            let value = trajectory.partial(state_index, parameter_index, last).unwrap_or(0.0);
            entries.push(format!(
                "    {{\"state\": {}, \"parameter\": {}, \"value\": {:.17e}}}",
                json_string(state.as_str()),
                json_string(parameter.as_str()),
                value
            ));
        }
    }
    let _ = writeln!(out, "{}", entries.join(",\n"));
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn help_documents_the_required_flags() {
        let help = help();
        assert!(help.contains("--parameters"));
        assert!(help.contains("--initial"));
    }

    #[test]
    fn initial_defaults_missing_states_to_zero() {
        let states = vec![id("x"), id("y")];
        let initial = resolve_initial(&states, &[(id("y"), 3.0)]).unwrap();
        assert_eq!(initial, vec![0.0, 3.0]);
    }

    #[test]
    fn initial_rejects_unknown_state() {
        let states = vec![id("x")];
        assert!(resolve_initial(&states, &[(id("z"), 1.0)]).unwrap_err().contains("not a state"));
    }
}
