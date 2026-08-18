//! `lawsynth estimate` — state-estimator (observer / Kalman) design.
//!
//! A discovered world gives a nonlinear field `ẋ = f(x)`. This command linearizes
//! it at a located fixed point to get `A = ∂f/∂x`, forms the output map `C` from
//! the user-named measured states, and designs a state estimator that
//! reconstructs the *full* state from those *partial* measurements. The numerics
//! come from [`lawsynth_estimate`], which places the error poles by duality with
//! `lawsynth-feedback`.
//!
//! # How `A` and `C` come from a world (honest assumptions)
//!
//! - `A` is the analytic Jacobian of the world's field evaluated at the **first**
//!   fixed point located inside `--box` (the count of fixed points found is
//!   reported, so the choice is auditable). Estimation is of that *linearization*;
//!   it is exact only near the equilibrium.
//! - `C` selects the measured states named by `--measure`: one row per measured
//!   state, a single `1` in that state's column. Pole placement
//!   (Ackermann's formula) is single-output, so it needs exactly one measured
//!   state; the multi-output Kalman filter (`--kalman`) accepts several.
//! - The input map `B` never enters estimator design (the gain shapes `A − L C`),
//!   so it is not required here.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_estimate::{Matrix, Observer, ObserverMethod, design_observer, kalman_filter};
use lawsynth_report::format_number;

use crate::analysis::{
    classification_label, json_string, linearize_first, matrix_json, parse_identifiers,
    parse_poles, parse_positive, parse_search_box, parse_usize, render_complex, render_coordinates,
};

/// Help text for `lawsynth estimate`.
pub fn help() -> String {
    "lawsynth estimate WORLD.lsworld --box LOW:HIGH[,LOW:HIGH...] \
--measure NAME[,NAME...] [--poles P[,P...] | --kalman [--process-var V] \
[--measurement-var V]] [--grid N] [--json]\n\n\
Linearizes the world's field at the first fixed point located inside the search \
box (A = df/dx there), builds the output map C from the measured states, and \
designs a state estimator. With --poles it places the error poles of A - L C by \
Ackermann's formula (single measured state only); each pole is REAL or REAL:IMAG \
and there must be one per state. With --kalman it designs the steady-state Kalman \
gain (process covariance Q = process-var * I, measurement covariance R = \
measurement-var * I, defaults 1). Reports the gain L, the error-dynamics \
eigenvalues, and whether the estimator converges. --json emits a stable report."
        .to_owned()
}

/// Runs the `estimate` command.
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

    let mut search_box = None;
    let mut measured = None;
    let mut poles = None;
    let mut kalman = false;
    let mut process_var = 1.0;
    let mut measurement_var = 1.0;
    let mut grid = None;
    let mut as_json = false;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--kalman" {
            kalman = true;
            index += 1;
            continue;
        }
        if option == "--json" {
            as_json = true;
            index += 1;
            continue;
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--box" => search_box = Some(parse_search_box(value)?),
            "--measure" => measured = Some(parse_identifiers(value)?),
            "--poles" => poles = Some(parse_poles(value)?),
            "--process-var" => process_var = parse_positive(value, "--process-var")?,
            "--measurement-var" => measurement_var = parse_positive(value, "--measurement-var")?,
            "--grid" => grid = Some(parse_usize(value, "--grid")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = lawsynth_bundle::read_world(bundle).map_err(|error| error.to_string())?;
    let search_box =
        search_box.ok_or_else(|| "--box is required to locate a fixed point".to_owned())?;
    let measured = measured.ok_or_else(|| "--measure NAME[,NAME...] is required".to_owned())?;
    if kalman && poles.is_some() {
        return Err("choose either --poles (pole placement) or --kalman, not both".to_owned());
    }

    let linear = linearize_first(&world, search_box, grid)?;
    let output_map = build_output_map(&linear.states, &measured)?;

    let observer = if kalman {
        let process = scaled_identity(linear.states.len(), process_var);
        let measurement = scaled_identity(measured.len(), measurement_var);
        kalman_filter(&linear.a, &output_map, &process, &measurement)
            .map_err(|error| error.to_string())?
    } else {
        let poles = poles.ok_or_else(|| {
            "pole placement needs --poles P[,P...] (one per state), or use --kalman".to_owned()
        })?;
        if measured.len() != 1 {
            return Err(
                "pole placement is single-output: pass exactly one --measure state, or use --kalman"
                    .to_owned(),
            );
        }
        if poles.len() != linear.states.len() {
            return Err(format!(
                "--poles needs one pole per state ({} state(s)), got {}",
                linear.states.len(),
                poles.len()
            ));
        }
        design_observer(&linear.a, &output_map, &poles).map_err(|error| error.to_string())?
    };

    if as_json {
        Ok(render_json(bundle, &linear, &measured, &observer))
    } else {
        Ok(render_text(bundle, &linear, &measured, &observer))
    }
}

/// Builds the output map `C`: one row per measured state, a single `1` in that
/// state's column, in the order the states were named.
fn build_output_map(states: &[Identifier], measured: &[Identifier]) -> Result<Matrix, String> {
    let mut c = Matrix::zeros(measured.len(), states.len());
    for (row, name) in measured.iter().enumerate() {
        let column = states
            .iter()
            .position(|state| state == name)
            .ok_or_else(|| format!("--measure '{}' is not a state of the world", name.as_str()))?;
        c.set(row, column, 1.0);
    }
    Ok(c)
}

/// A `scale * I` matrix of order `n`.
fn scaled_identity(n: usize, scale: f64) -> Matrix {
    let mut matrix = Matrix::zeros(n, n);
    for index in 0..n {
        matrix.set(index, index, scale);
    }
    matrix
}

/// A human-readable name for the design method.
fn method_label(method: ObserverMethod) -> &'static str {
    match method {
        ObserverMethod::PolePlacement => "pole placement (Ackermann, single-output)",
        ObserverMethod::Kalman => "steady-state Kalman filter",
    }
}

/// Renders a gain matrix `L` as indented rows of numbers.
fn render_gain(gain: &Matrix) -> String {
    let mut out = String::new();
    for row in 0..gain.rows() {
        let cells: Vec<String> =
            (0..gain.cols()).map(|col| format_number(gain.get(row, col))).collect();
        let _ = writeln!(out, "    [ {} ]", cells.join(", "));
    }
    out
}

/// Human-facing report.
fn render_text(
    bundle: &str,
    linear: &crate::analysis::Linearization,
    measured: &[Identifier],
    observer: &Observer,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "State estimator design for {bundle}");
    let states: Vec<&str> = linear.states.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states:   {}", states.join(", "));
    let _ = writeln!(
        out,
        "  fixed point: ({}) [{}]  ({} of {} located, using the first)",
        render_coordinates(&linear.states, &linear.coordinates),
        classification_label(linear.classification),
        linear.points_found,
        linear.points_found.max(1)
    );
    let measured_names: Vec<&str> = measured.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  measured: {} (C selects these states)", measured_names.join(", "));
    let _ = writeln!(out, "  method:   {}", method_label(observer.method));
    out.push('\n');

    let _ = writeln!(out, "Observer gain L ({}x{}):", observer.states(), observer.outputs());
    out.push_str(&render_gain(&observer.gain));
    out.push('\n');

    let _ = writeln!(out, "Error dynamics A - L C eigenvalues:");
    for pole in &observer.error_poles {
        let _ = writeln!(out, "    {}", render_complex(pole));
    }
    let convergent = observer.is_convergent(1e-9);
    let _ = writeln!(
        out,
        "  convergent: {} (all Re < 0 means the estimate x_hat -> x)",
        if convergent { "yes" } else { "no" }
    );

    if let Some(covariance) = &observer.covariance {
        out.push('\n');
        let _ = writeln!(
            out,
            "Steady-state error covariance P ({}x{}):",
            covariance.rows(),
            covariance.cols()
        );
        out.push_str(&render_gain(covariance));
    }

    out.push('\n');
    let _ = writeln!(
        out,
        "note: estimation is of the linearization A = df/dx at the located fixed point; \
C selects the measured states; the input map B does not enter estimator design."
    );
    out
}

/// Stable, machine-readable report.
fn render_json(
    bundle: &str,
    linear: &crate::analysis::Linearization,
    measured: &[Identifier],
    observer: &Observer,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let states: Vec<String> =
        linear.states.iter().map(|state| json_string(state.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", states.join(", "));
    let coordinates: Vec<String> =
        linear.coordinates.iter().map(|value| format!("{value:.17e}")).collect();
    let _ = writeln!(out, "  \"fixed_point\": [{}],", coordinates.join(", "));
    let _ = writeln!(out, "  \"fixed_points_found\": {},", linear.points_found);
    let measured_names: Vec<String> =
        measured.iter().map(|name| json_string(name.as_str())).collect();
    let _ = writeln!(out, "  \"measured\": [{}],", measured_names.join(", "));
    let method = match observer.method {
        ObserverMethod::PolePlacement => "pole_placement",
        ObserverMethod::Kalman => "kalman",
    };
    let _ = writeln!(out, "  \"method\": {},", json_string(method));
    let _ = writeln!(out, "  \"gain\": {},", matrix_json(&observer.gain));
    let poles: Vec<String> = observer
        .error_poles
        .iter()
        .map(|pole| format!("{{\"re\": {:.17e}, \"im\": {:.17e}}}", pole.re, pole.im))
        .collect();
    let _ = writeln!(out, "  \"error_poles\": [{}],", poles.join(", "));
    let _ = writeln!(out, "  \"convergent\": {},", observer.is_convergent(1e-9));
    match &observer.covariance {
        Some(covariance) => {
            let _ = writeln!(out, "  \"covariance\": {}", matrix_json(covariance));
        }
        None => {
            let _ = writeln!(out, "  \"covariance\": null");
        }
    }
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
    fn help_documents_the_flags() {
        let help = help();
        assert!(help.contains("--measure"));
        assert!(help.contains("--poles"));
        assert!(help.contains("--kalman"));
    }

    #[test]
    fn output_map_selects_named_states() {
        let states = vec![id("x"), id("y"), id("z")];
        let c = build_output_map(&states, &[id("z")]).unwrap();
        assert_eq!((c.rows(), c.cols()), (1, 3));
        assert_eq!(c.get(0, 2), 1.0);
        assert_eq!(c.get(0, 0), 0.0);
    }

    #[test]
    fn output_map_rejects_unknown_state() {
        let states = vec![id("x")];
        assert!(build_output_map(&states, &[id("q")]).unwrap_err().contains("not a state"));
    }

    #[test]
    fn scaled_identity_is_diagonal() {
        let matrix = scaled_identity(2, 3.0);
        assert_eq!(matrix.get(0, 0), 3.0);
        assert_eq!(matrix.get(1, 1), 3.0);
        assert_eq!(matrix.get(0, 1), 0.0);
    }
}
