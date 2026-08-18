//! `lawsynth control` — controlled (SINDYc) discovery of forced systems.
//!
//! Ordinary `discover` fits an autonomous field `ẋ = f(x)`. Many real systems are
//! *forced*: `ẋ = f(x, u)`, where `u(t)` are exogenous, measured control inputs.
//! This command designates which dataset columns are states and which are
//! controls, runs [`lawsynth_control::discover_controlled`] (SINDYc), and prints
//! the fitted per-state equations over the augmented library `Θ(x, u)`.
//!
//! # Why a dedicated subcommand (not `discover --control`)
//!
//! A controlled model is a distinct type ([`ControlledModel`]) that does **not**
//! serialize to a `.lsworld` bundle the way `discover`'s output does: controls
//! appear only inside library terms, there is exactly one equation per state and
//! none for a control, and forward simulation needs an external control signal.
//! Overloading `discover` (already the largest command, and world-bundle centric)
//! would blur that boundary. A separate `control` command keeps each command's
//! contract honest and its output type clear.
//!
//! With `--validate`, the command additionally rolls the model forward under the
//! dataset's own control columns and scores it against the observed states
//! ([`lawsynth_control::validate_controlled`]), reporting per-state and aggregate
//! R²/RMSE. This is an **in-sample** score (same data used to fit) and is labelled
//! as such.

use std::fmt::Write as _;

use lawsynth_control::{
    ControlConfig, ControlScore, ControlSpec, ControlledModel, ValidationConfig,
    discover_controlled, validate_controlled,
};
use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_report::format_number;

use crate::read_numeric_dataset;

/// Help text for `lawsynth control`.
pub fn help() -> String {
    "lawsynth control OBSERVATIONS.{csv,tsv,parquet} --time COLUMN \
--state NAME[,NAME...] --control NAME[,NAME...] [--degree N] [--threshold V] \
[--validate] [--json]\n\n\
Discovers a forced model dx/dt = f(x, u) (SINDYc): state columns have their \
derivatives estimated and fitted; control columns enter the candidate library but \
are never differentiated and never predicted. Prints one equation per state over \
the augmented library. --validate rolls the model forward under the dataset's own \
controls and reports in-sample R²/RMSE. --json emits a stable machine-readable model."
        .to_owned()
}

/// Runs the `control` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let Some(input) = arguments.first() else {
        return Err(help());
    };
    if input.starts_with('-') {
        return Err(help());
    }

    let mut time_column = None;
    let mut states = None;
    let mut controls = None;
    let mut degree = None;
    let mut threshold = None;
    let mut validate = false;
    let mut as_json = false;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--validate" || option == "--json" {
            match option {
                "--validate" => validate = true,
                "--json" => as_json = true,
                _ => unreachable!(),
            }
            index += 1;
            continue;
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--time" => time_column = Some(value.clone()),
            "--state" => states = Some(parse_identifiers(value)?),
            "--control" => controls = Some(parse_identifiers(value)?),
            "--degree" => degree = Some(parse_usize(value, "--degree")?),
            "--threshold" => threshold = Some(parse_number(value, "--threshold")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let time_column = time_column.ok_or_else(|| "--time COLUMN is required".to_owned())?;
    let states = states.ok_or_else(|| "--state NAME[,NAME...] is required".to_owned())?;
    let controls = controls.ok_or_else(|| "--control NAME[,NAME...] is required".to_owned())?;

    let dataset = read_numeric_dataset(input, &time_column)?;
    let spec = ControlSpec::new(states, controls).map_err(|error| error.to_string())?;

    let mut config = ControlConfig::default();
    if let Some(degree) = degree {
        config.features.polynomial_degree = degree;
    }
    if let Some(threshold) = threshold {
        config.sparse.threshold = threshold;
    }

    let model = discover_controlled(&dataset, &spec, &config).map_err(|error| error.to_string())?;

    let score = if validate {
        Some(
            validate_controlled(&model, &dataset, &spec, &ValidationConfig::default())
                .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };

    if as_json {
        Ok(render_json(input, &model, score.as_ref()))
    } else {
        Ok(render_text(input, &dataset, &model, score.as_ref()))
    }
}

/// Renders `coefficient*term + ...` for one fitted equation, in library order.
fn render_equation(equation_terms: &[(&str, f64)]) -> String {
    if equation_terms.is_empty() {
        return "0".to_owned();
    }
    equation_terms
        .iter()
        .map(|(term, coefficient)| format!("{}*{term}", format_number(*coefficient)))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Human-facing report.
fn render_text(
    source: &str,
    dataset: &Dataset,
    model: &ControlledModel,
    score: Option<&ControlScore>,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Controlled discovery from {source}");
    let states: Vec<&str> = model.states.iter().map(Identifier::as_str).collect();
    let controls: Vec<&str> = model.controls.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states:   {}", states.join(", "));
    let _ = writeln!(out, "  controls: {}", controls.join(", "));
    let _ = writeln!(out, "  samples:  {}", dataset.time().values().len());
    let _ = writeln!(out, "  library:  {} augmented term(s)", model.library_terms.len());
    out.push('\n');

    let _ = writeln!(out, "Discovered model dx/dt = f(x, u):");
    for equation in &model.equations {
        let terms = equation.active_terms(&model.library_terms);
        let _ = writeln!(out, "  d/dt {} = {}", equation.state.as_str(), render_equation(&terms));
        let _ = writeln!(
            out,
            "      active terms: {}, residual SS: {}",
            terms.len(),
            format_number(equation.residual_sum_squares)
        );
    }

    if let Some(score) = score {
        out.push('\n');
        let _ = writeln!(out, "In-sample validation (rollout vs. observed states):");
        for state_score in &score.per_state {
            let _ = writeln!(
                out,
                "  {:<12} R2={}  RMSE={}",
                state_score.state.as_str(),
                format_number(state_score.r_squared),
                format_number(state_score.rmse)
            );
        }
        let _ = writeln!(
            out,
            "  aggregate    R2={}  RMSE={}",
            format_number(score.aggregate_r_squared),
            format_number(score.aggregate_rmse)
        );
        let _ = writeln!(
            out,
            "  note: in-sample (same data fitted); open-loop rollout error grows with horizon."
        );
    }
    out
}

/// Stable, machine-readable model + optional validation score.
fn render_json(source: &str, model: &ControlledModel, score: Option<&ControlScore>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"source\": {},", json_string(source));
    let states: Vec<String> =
        model.states.iter().map(|state| json_string(state.as_str())).collect();
    let controls: Vec<String> =
        model.controls.iter().map(|control| json_string(control.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", states.join(", "));
    let _ = writeln!(out, "  \"controls\": [{}],", controls.join(", "));
    let _ = writeln!(out, "  \"equations\": [");
    for (number, equation) in model.equations.iter().enumerate() {
        let terms = equation.active_terms(&model.library_terms);
        let rendered: Vec<String> = terms
            .iter()
            .map(|(term, coefficient)| {
                format!(
                    "{{\"term\": {}, \"coefficient\": {:.17e}}}",
                    json_string(term),
                    coefficient
                )
            })
            .collect();
        let _ = writeln!(out, "    {{");
        let _ = writeln!(out, "      \"state\": {},", json_string(equation.state.as_str()));
        let _ = writeln!(
            out,
            "      \"residual_sum_squares\": {:.17e},",
            equation.residual_sum_squares
        );
        let _ = writeln!(out, "      \"terms\": [{}]", rendered.join(", "));
        let terminator = if number + 1 == model.equations.len() { "    }" } else { "    }," };
        let _ = writeln!(out, "{terminator}");
    }
    let _ = writeln!(out, "  ],");
    match score {
        Some(score) => {
            let per_state: Vec<String> = score
                .per_state
                .iter()
                .map(|state_score| {
                    format!(
                        "{{\"state\": {}, \"r_squared\": {:.17e}, \"rmse\": {:.17e}}}",
                        json_string(state_score.state.as_str()),
                        state_score.r_squared,
                        state_score.rmse
                    )
                })
                .collect();
            let _ = writeln!(out, "  \"validation\": {{");
            let _ = writeln!(out, "    \"in_sample\": true,");
            let _ = writeln!(out, "    \"per_state\": [{}],", per_state.join(", "));
            let _ =
                writeln!(out, "    \"aggregate_r_squared\": {:.17e},", score.aggregate_r_squared);
            let _ = writeln!(out, "    \"aggregate_rmse\": {:.17e}", score.aggregate_rmse);
            let _ = writeln!(out, "  }}");
        }
        None => {
            let _ = writeln!(out, "  \"validation\": null");
        }
    }
    let _ = writeln!(out, "}}");
    out
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn parse_identifiers(value: &str) -> Result<Vec<Identifier>, String> {
    let identifiers = value
        .split(',')
        .map(|item| Identifier::new(item.trim()).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if identifiers.is_empty() {
        Err("expected at least one identifier".to_owned())
    } else {
        Ok(identifiers)
    }
}

fn parse_number(value: &str, flag: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}' for {flag}"))?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(format!("value '{value}' for {flag} must be finite"))
    }
}

fn parse_usize(value: &str, flag: &str) -> Result<usize, String> {
    value.parse().map_err(|_| format!("invalid count '{value}' for {flag}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_mentions_the_flags() {
        let help = help();
        assert!(help.contains("--state"));
        assert!(help.contains("--control"));
        assert!(help.contains("--validate"));
    }

    #[test]
    fn requires_state_and_control() {
        let error = run(&["data.csv".to_owned(), "--time".to_owned(), "t".to_owned()]).unwrap_err();
        assert!(error.contains("--state"));
    }

    #[test]
    fn renders_an_equation_line() {
        let terms = vec![("x", 2.0_f64), ("x*u", -1.0_f64)];
        assert_eq!(render_equation(&terms), "2*x + -1*x*u");
        assert_eq!(render_equation(&[]), "0");
    }
}
