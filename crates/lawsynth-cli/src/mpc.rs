//! `lawsynth mpc` — successive-linearization (LQR) model-predictive control.
//!
//! Loads a continuous world whose laws reference one or more **control** symbols,
//! reads them as a forced field `ẋ = f(x, u)`, and drives the state to a setpoint
//! with the deterministic controller of [`lawsynth_mpc::mpc_control`]. At each step
//! it relinearizes about the current point, designs a local LQR gain, applies the
//! first (optionally saturated) move, and RK4-advances the true nonlinear plant.
//! It reports the final state and the final error norm, and — under `--json` — the
//! full state/control trajectory.
//!
//! # Honest limits
//!
//! This is *successive-linearization LQR-MPC*, not a constrained QP-MPC: the local
//! LQR needs a **stabilizable** linearization, saturation is applied by clamping
//! (not a constraint-optimal projection), optimality is only local to each
//! linearization, and there is no horizon/feasibility guarantee. When the LQR
//! design fails (e.g. an unstabilizable linearization or a non-positive-definite
//! `R`), the engine error is surfaced verbatim rather than hidden.

use std::fmt::Write as _;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_mpc::{Matrix, MpcConfig, MpcTrajectory, mpc_control};
use lawsynth_report::format_number;

use crate::analysis::{
    autonomous_fields, json_string, parse_identifiers, parse_positive, parse_state_vector,
    parse_usize,
};

/// The default fixed integration step.
const DEFAULT_DT: f64 = 0.05;
/// The default closed-loop horizon in control steps.
const DEFAULT_STEPS: usize = 200;

/// Help text for `lawsynth mpc`.
pub fn help() -> String {
    "lawsynth mpc WORLD.lsworld --control NAME[,NAME...] \
--setpoint NAME=VALUE[,NAME=VALUE...] --initial NAME=VALUE[,NAME=VALUE...] \
[--dt DT] [--steps N] [--q W] [--r W] [--u-min V] [--u-max V] [--json]\n\n\
Regulates a world's forced field dx/dt = f(x, u) to the setpoint by \
successive-linearization LQR-MPC: at each step it relinearizes, designs a local \
LQR gain from weights Q = q*I and R = r*I, applies the first (clamped) move, and \
RK4-advances the true nonlinear plant. Reports the final state and final error \
norm. --q/--r scale the identity state/control weights; --u-min/--u-max saturate \
every control channel. The local LQR needs a stabilizable linearization; if the \
design fails the engine error is surfaced. --json adds the state and control \
trajectory."
        .to_owned()
}

/// Runs the `mpc` command.
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

    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();

    let mut controls = None;
    let mut setpoint = None;
    let mut initial = None;
    let mut dt = DEFAULT_DT;
    let mut steps = DEFAULT_STEPS;
    let mut q = 1.0;
    let mut r = 1.0;
    let mut u_min = None;
    let mut u_max = None;
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
            "--control" => controls = Some(parse_identifiers(value)?),
            "--setpoint" => setpoint = Some(parse_state_vector(value, &states, "--setpoint")?),
            "--initial" => initial = Some(parse_state_vector(value, &states, "--initial")?),
            "--dt" => dt = parse_positive(value, "--dt")?,
            "--steps" => steps = parse_usize(value, "--steps")?,
            "--q" => q = parse_positive(value, "--q")?,
            "--r" => r = parse_positive(value, "--r")?,
            "--u-min" => u_min = Some(parse_number(value, "--u-min")?),
            "--u-max" => u_max = Some(parse_number(value, "--u-max")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let controls = controls.ok_or_else(|| "--control NAME[,NAME...] is required".to_owned())?;
    let setpoint = setpoint.ok_or_else(|| "--setpoint is required".to_owned())?;
    let initial = initial.ok_or_else(|| "--initial is required".to_owned())?;
    let n = states.len();
    let m = controls.len();

    let fields = autonomous_fields(&world);
    let state_weight = scaled_identity(n, q);
    let control_weight = scaled_identity(m, r);
    let mut config =
        MpcConfig::new(initial, setpoint.clone(), state_weight, control_weight, dt, steps);
    if let Some((lo, hi)) = saturation_bounds(u_min, u_max, m)? {
        config = config.with_saturation(lo, hi);
    }

    let trajectory =
        mpc_control(&fields, &states, &controls, &config).map_err(|error| error.to_string())?;

    if as_json {
        Ok(render_json(bundle, &states, &controls, &setpoint, &trajectory))
    } else {
        Ok(render_text(bundle, &states, &controls, &setpoint, &trajectory))
    }
}

/// Per-channel control saturation bounds `(lower, upper)`, one entry per control.
type SaturationBounds = (Vec<f64>, Vec<f64>);

/// An `n × n` diagonal matrix with `scale` on the diagonal (`scale · I`).
fn scaled_identity(n: usize, scale: f64) -> Matrix {
    let rows: Vec<Vec<f64>> =
        (0..n).map(|i| (0..n).map(|j| if i == j { scale } else { 0.0 }).collect()).collect();
    // `n >= 1` here (a world has at least one state, `--control` at least one
    // identifier), so `from_rows` cannot fail on an empty matrix.
    Matrix::from_rows(rows).expect("a scaled identity is a valid square matrix")
}

/// Builds the per-channel saturation bounds from the scalar flags, applied to
/// every control channel. Saturation is opt-in: `None` when neither flag is set.
/// Both bounds must be given together (the engine requires finite two-sided
/// bounds) and the lower must not exceed the upper.
fn saturation_bounds(
    u_min: Option<f64>,
    u_max: Option<f64>,
    m: usize,
) -> Result<Option<SaturationBounds>, String> {
    match (u_min, u_max) {
        (None, None) => Ok(None),
        (Some(lo), Some(hi)) => {
            if lo > hi {
                return Err("--u-min must not exceed --u-max".to_owned());
            }
            Ok(Some((vec![lo; m], vec![hi; m])))
        }
        _ => Err("provide both --u-min and --u-max (two-sided saturation) or neither".to_owned()),
    }
}

/// Human-facing report.
fn render_text(
    bundle: &str,
    states: &[Identifier],
    controls: &[Identifier],
    setpoint: &[f64],
    trajectory: &MpcTrajectory,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Model-predictive control of {bundle}");
    let state_names: Vec<&str> = states.iter().map(Identifier::as_str).collect();
    let control_names: Vec<&str> = controls.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states:   {}", state_names.join(", "));
    let _ = writeln!(out, "  controls: {}", control_names.join(", "));
    let _ = writeln!(out, "  steps:    {}", trajectory.controls().len());
    let target = states
        .iter()
        .zip(setpoint)
        .map(|(state, value)| format!("{}={}", state.as_str(), format_number(*value)))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(out, "  setpoint: {target}");
    out.push('\n');

    let final_state = states
        .iter()
        .zip(trajectory.final_state())
        .map(|(state, value)| format!("{}={}", state.as_str(), format_number(*value)))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = writeln!(out, "  final state: {final_state}");
    match trajectory.final_error_norm(setpoint) {
        Some(error) => {
            let _ = writeln!(out, "  final error norm: {}", format_number(error));
        }
        None => {
            let _ = writeln!(out, "  final error norm: (unavailable)");
        }
    }
    let _ = writeln!(
        out,
        "note: successive-linearization LQR-MPC \u{2014} local optimality only, and \
the linearization must be stabilizable."
    );
    out
}

/// Stable, machine-readable report, including the full trajectory.
fn render_json(
    bundle: &str,
    states: &[Identifier],
    controls: &[Identifier],
    setpoint: &[f64],
    trajectory: &MpcTrajectory,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let state_names: Vec<String> = states.iter().map(|s| json_string(s.as_str())).collect();
    let control_names: Vec<String> = controls.iter().map(|c| json_string(c.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", state_names.join(", "));
    let _ = writeln!(out, "  \"controls\": [{}],", control_names.join(", "));
    let setpoint_cells: Vec<String> =
        setpoint.iter().map(|value| format!("{value:.17e}")).collect();
    let _ = writeln!(out, "  \"setpoint\": [{}],", setpoint_cells.join(", "));

    let final_cells: Vec<String> =
        trajectory.final_state().iter().map(|value| format!("{value:.17e}")).collect();
    let _ = writeln!(out, "  \"final_state\": [{}],", final_cells.join(", "));
    match trajectory.final_error_norm(setpoint) {
        Some(error) => {
            let _ = writeln!(out, "  \"final_error_norm\": {error:.17e},");
        }
        None => {
            let _ = writeln!(out, "  \"final_error_norm\": null,");
        }
    }

    let _ = writeln!(out, "  \"state_trajectory\": [{}],", rows_json(trajectory.states()));
    let _ = writeln!(out, "  \"control_trajectory\": [{}]", rows_json(trajectory.controls()));
    let _ = writeln!(out, "}}");
    out
}

/// Renders a sequence of numeric rows as a JSON array of arrays of 17-digit floats.
fn rows_json(rows: &[Vec<f64>]) -> String {
    let rendered: Vec<String> = rows
        .iter()
        .map(|row| {
            let cells: Vec<String> = row.iter().map(|value| format!("{value:.17e}")).collect();
            format!("[{}]", cells.join(", "))
        })
        .collect();
    rendered.join(", ")
}

/// Parses a finite floating-point value, tagging the flag for error messages.
fn parse_number(value: &str, flag: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}' for {flag}"))?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(format!("value '{value}' for {flag} must be finite"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_flags() {
        let help = help();
        assert!(help.contains("--control"));
        assert!(help.contains("--setpoint"));
        assert!(help.contains("stabilizable"));
    }

    #[test]
    fn scaled_identity_is_diagonal() {
        let matrix = scaled_identity(2, 3.0);
        assert_eq!(matrix.get(0, 0), 3.0);
        assert_eq!(matrix.get(1, 1), 3.0);
        assert_eq!(matrix.get(0, 1), 0.0);
        assert_eq!(matrix.get(1, 0), 0.0);
    }

    #[test]
    fn saturation_rejects_inverted_bounds() {
        assert!(saturation_bounds(Some(1.0), Some(-1.0), 1).is_err());
        let (lo, hi) = saturation_bounds(Some(-2.0), Some(2.0), 2).unwrap().unwrap();
        assert_eq!(lo, vec![-2.0, -2.0]);
        assert_eq!(hi, vec![2.0, 2.0]);
    }

    #[test]
    fn saturation_is_opt_in_and_two_sided() {
        assert_eq!(saturation_bounds(None, None, 1).unwrap(), None);
        assert!(saturation_bounds(None, Some(5.0), 1).is_err());
        assert!(saturation_bounds(Some(-5.0), None, 1).is_err());
    }
}
