//! `lawsynth lyapunov` — Lyapunov-spectrum (chaos) diagnostic of a world.
//!
//! Loads a continuous world, reads its laws as an autonomous vector field
//! `ẋ = f(x)`, and runs the deterministic Benettin/QR estimator of
//! [`lawsynth_lyapunov::lyapunov_spectrum`] from a caller-supplied initial
//! condition. It reports the full spectrum (sorted descending), the largest
//! exponent, the exponent sum (time-averaged divergence — the tightest quantity),
//! and the Kaplan–Yorke dimension, and states plainly whether the largest exponent
//! is positive (the signature of chaos).
//!
//! The spectrum is a **time-averaged estimate**: its accuracy depends on the run
//! length, the step `dt`, and the reorthonormalization interval, and the initial
//! condition should lie in the basin of the attractor whose spectrum is sought.
//! The command surfaces this caveat rather than presenting the numbers as exact.

use std::fmt::Write as _;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_lyapunov::{LyapunovConfig, LyapunovReport, lyapunov_spectrum};
use lawsynth_report::format_number;

use crate::analysis::{
    autonomous_fields, json_string, parse_positive, parse_state_vector, parse_usize,
};

/// Help text for `lawsynth lyapunov`.
pub fn help() -> String {
    "lawsynth lyapunov WORLD.lsworld --initial NAME=VALUE[,NAME=VALUE...] \
[--dt DT] [--steps N] [--reorth K] [--transient F] [--json]\n\n\
Estimates the Lyapunov spectrum of a world's autonomous vector field from the \
given initial condition (one NAME=VALUE per state) with the deterministic \
Benettin/QR method. Reports the spectrum (sorted descending), the largest \
exponent, the exponent sum (time-averaged divergence), and the Kaplan-Yorke \
dimension, and states whether the largest exponent is positive (chaos). This is a \
time-averaged estimate: its accuracy grows with --steps and shrinks with --dt, and \
the initial condition should sit in the target attractor's basin. --json emits a \
stable machine-readable report."
        .to_owned()
}

/// Runs the `lyapunov` command.
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

    let mut initial = None;
    let mut dt = None;
    let mut steps = None;
    let mut reorth = None;
    let mut transient = None;
    let mut as_json = false;

    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();

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
            "--initial" => initial = Some(parse_state_vector(value, &states, "--initial")?),
            "--dt" => dt = Some(parse_positive(value, "--dt")?),
            "--steps" => steps = Some(parse_usize(value, "--steps")?),
            "--reorth" => reorth = Some(parse_usize(value, "--reorth")?),
            "--transient" => transient = Some(parse_transient(value)?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let initial = initial.ok_or_else(|| {
        format!("--initial is required (one NAME=VALUE per state, {} state(s))", states.len())
    })?;

    let fields = autonomous_fields(&world);
    let mut config = LyapunovConfig::default();
    if let Some(dt) = dt {
        config = config.with_step(dt);
    }
    if let Some(steps) = steps {
        config = config.with_steps(steps);
    }
    if let Some(reorth) = reorth {
        config = config.with_reorthonormalization_interval(reorth);
    }
    if let Some(transient) = transient {
        config = config.with_transient_fraction(transient);
    }

    let report = lyapunov_spectrum(&fields, &states, &initial, &config)
        .map_err(|error| error.to_string())?;

    if as_json {
        Ok(render_json(bundle, &states, &report))
    } else {
        Ok(render_text(bundle, &states, &initial, &report))
    }
}

/// Whether the largest exponent is positive — the signature of chaos.
fn is_chaotic(report: &LyapunovReport) -> bool {
    report.largest() > 0.0
}

/// Human-facing report.
fn render_text(
    bundle: &str,
    states: &[Identifier],
    initial: &[f64],
    report: &LyapunovReport,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Lyapunov spectrum of {bundle}");
    let names: Vec<&str> = states.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states:  {}", names.join(", "));
    let start: Vec<String> = states
        .iter()
        .zip(initial)
        .map(|(state, value)| format!("{}={}", state.as_str(), format_number(*value)))
        .collect();
    let _ = writeln!(out, "  initial: {}", start.join(", "));
    let _ = writeln!(
        out,
        "  window:  {} time units (post-transient)",
        format_number(report.integration_time())
    );
    out.push('\n');

    let exponents: Vec<String> =
        report.exponents().iter().map(|value| format_number(*value)).collect();
    let _ = writeln!(out, "  spectrum:        {}", exponents.join(", "));
    let _ = writeln!(out, "  largest:         {}", format_number(report.largest()));
    let _ = writeln!(out, "  sum (divergence): {}", format_number(report.sum()));
    let _ = writeln!(out, "  kaplan-yorke dim: {}", format_number(report.kaplan_yorke_dimension()));
    out.push('\n');

    if is_chaotic(report) {
        let _ = writeln!(
            out,
            "The largest exponent is positive: nearby trajectories separate \
exponentially \u{2014} the signature of chaos."
        );
    } else {
        let _ = writeln!(
            out,
            "The largest exponent is not positive: no chaos detected (trajectories \
do not separate exponentially in this estimate)."
        );
    }
    let _ = writeln!(
        out,
        "note: a time-averaged estimate \u{2014} lengthen --steps or shrink --dt to \
sharpen it; the sum is the tightest quantity."
    );
    out
}

/// Stable, machine-readable report. Floats use the full 17-digit form so the JSON
/// is a faithful, deterministic image of the report.
fn render_json(bundle: &str, states: &[Identifier], report: &LyapunovReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let names: Vec<String> = states.iter().map(|state| json_string(state.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", names.join(", "));
    let exponents: Vec<String> =
        report.exponents().iter().map(|value| format!("{value:.17e}")).collect();
    let _ = writeln!(out, "  \"exponents\": [{}],", exponents.join(", "));
    let _ = writeln!(out, "  \"largest\": {:.17e},", report.largest());
    let _ = writeln!(out, "  \"sum\": {:.17e},", report.sum());
    let _ =
        writeln!(out, "  \"kaplan_yorke_dimension\": {:.17e},", report.kaplan_yorke_dimension());
    let _ = writeln!(out, "  \"integration_time\": {:.17e},", report.integration_time());
    let _ = writeln!(out, "  \"chaotic\": {}", is_chaotic(report));
    let _ = writeln!(out, "}}");
    out
}

/// Parses the transient fraction, requiring a finite value in `[0, 1)`.
fn parse_transient(value: &str) -> Result<f64, String> {
    let number: f64 =
        value.parse().map_err(|_| format!("invalid number '{value}' for --transient"))?;
    if !number.is_finite() || !(0.0..1.0).contains(&number) {
        return Err("--transient must be a finite fraction in [0, 1)".to_owned());
    }
    Ok(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_flags() {
        let help = help();
        assert!(help.contains("--initial"));
        assert!(help.contains("--steps"));
        assert!(help.contains("chaos"));
    }

    #[test]
    fn transient_must_be_a_valid_fraction() {
        assert!(parse_transient("0.1").is_ok());
        assert!(parse_transient("1.0").is_err());
        assert!(parse_transient("-0.1").is_err());
        assert!(parse_transient("x").is_err());
    }
}
