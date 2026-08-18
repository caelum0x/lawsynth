//! `lawsynth koopman` — Koopman / DMD linear-operator discovery from data.
//!
//! Ordinary `discover` fits a *symbolic nonlinear* law `ẋ = f(x)` and serializes
//! a `.lsworld` world. Dynamic Mode Decomposition instead fits the best-fit
//! *linear* operator `A` that advances the state one step, `x' ≈ A x`, and
//! reports its spectrum. See [`lawsynth_koopman::dmd`].
//!
//! # Why a dedicated command (not `discover --method koopman`)
//!
//! DMD's output is a fundamentally different type from strong- and weak-form
//! discovery: a linear operator and its eigenvalues, not a symbolic law or a
//! world bundle. `discover` mandates `--output WORLD.lsworld` and writes a
//! serialized world; a DMD operator has no such representation. Overloading
//! `discover` with a method whose output cannot be a world would blur that
//! contract, so Koopman/DMD gets its own command with an honest, method-specific
//! summary. The strong- and weak-form methods — which both yield coefficient
//! laws — remain under `discover --method`.
//!
//! The recovered object is honestly a **linear (or lifted-linear) approximation**
//! of the dynamics, not a nonlinear symbolic law.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn};
use lawsynth_koopman::{Complex, DmdModel, dmd, snapshots_from_dataset};
use lawsynth_report::format_number;

use crate::analysis::{json_string, parse_identifiers, parse_usize, render_complex};
use crate::read_numeric_dataset;

/// Help text for `lawsynth koopman`.
pub fn help() -> String {
    "lawsynth koopman OBSERVATIONS.{csv,tsv,parquet} --state NAME[,NAME...] \
[--time COLUMN] [--rank R] [--json]\n\n\
Discovers the best-fit linear operator A with x' ≈ A x by Dynamic Mode \
Decomposition (DMD) over the dataset's snapshot pairs. Prints the discrete-time \
eigenvalues (per step, with modulus |λ|), the continuous-time eigenvalues \
ln(λ)/dt (growth rate in the real part, angular frequency in the imaginary \
part), and a spectral-radius stability summary. --rank truncates the SVD \
(default: full rank). --json emits the eigenvalues and singular values. DMD \
recovers a LINEAR (or lifted-linear) approximation, not a symbolic nonlinear \
law — it does not write a .lsworld bundle."
        .to_owned()
}

/// Runs the `koopman` command.
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

    let mut states = None;
    let mut time_column = None;
    let mut rank = None;
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
            "--state" => states = Some(parse_identifiers(value)?),
            "--time" => time_column = Some(value.clone()),
            "--rank" => rank = Some(parse_usize(value, "--rank")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let states = states.ok_or_else(|| "--state NAME[,NAME...] is required".to_owned())?;
    let time_column = time_column.unwrap_or_else(|| "time".to_owned());

    let dataset = read_numeric_dataset(input, &time_column)?;
    let dataset = select_states(&dataset, &states)?;

    let dt = mean_time_step(&dataset)?;
    let (x, x_prime) = snapshots_from_dataset(&dataset).map_err(|error| error.to_string())?;

    let state_dim = x.rows();
    let pairs = x.cols();
    let max_rank = state_dim.min(pairs);
    let rank = match rank {
        Some(0) => return Err("--rank must be >= 1".to_owned()),
        Some(requested) if requested > max_rank => {
            return Err(format!(
                "--rank {requested} exceeds the maximum usable rank {max_rank} \
(min of state count and snapshot pairs)"
            ));
        }
        Some(requested) => requested,
        None => max_rank,
    };

    let model = dmd(&x, &x_prime, rank).map_err(|error| error.to_string())?;

    // The state ordering used by the operator rows is the dataset's schema
    // (lexicographic) column order — the same order `snapshots_from_dataset` uses.
    let ordered_states: Vec<&str> = dataset.columns().keys().map(Identifier::as_str).collect();

    if as_json {
        Ok(render_json(input, &ordered_states, &model, dt))
    } else {
        Ok(render_text(input, &ordered_states, &model, dt, pairs))
    }
}

/// Restricts the dataset to the requested state columns, preserving the time
/// axis, so `--state` faithfully selects the observables fed to DMD.
fn select_states(dataset: &Dataset, states: &[Identifier]) -> Result<Dataset, String> {
    let mut columns = Vec::with_capacity(states.len());
    for state in states {
        let column = dataset.columns().get(state).ok_or_else(|| {
            format!("dataset has no column '{}' named in --state", state.as_str())
        })?;
        columns.push(NumericColumn::new(state.clone(), column.values.clone()));
    }
    Dataset::new(dataset.time().clone(), columns).map_err(|error| error.to_string())
}

/// The mean sampling interval `Δt`, used to map discrete eigenvalues to
/// continuous ones (`ln(λ)/Δt`). Requires at least two samples.
fn mean_time_step(dataset: &Dataset) -> Result<f64, String> {
    let time = dataset.time().values();
    if time.len() < 2 {
        return Err("need at least two time samples to form snapshot pairs".to_owned());
    }
    let dt = (time[time.len() - 1] - time[0]) / (time.len() - 1) as f64;
    if !dt.is_finite() || dt <= 0.0 {
        return Err("time axis must be increasing to define a positive Δt".to_owned());
    }
    Ok(dt)
}

/// The spectral radius `ρ = max|λ|`. A discrete linear system is asymptotically
/// stable exactly when `ρ < 1`.
fn spectral_radius(eigenvalues: &[Complex]) -> f64 {
    eigenvalues.iter().map(|value| value.abs()).fold(0.0_f64, f64::max)
}

/// Per-eigenvalue verdict from its discrete modulus.
fn mode_behaviour(modulus: f64) -> &'static str {
    if modulus < 1.0 - 1e-9 {
        "decaying"
    } else if modulus > 1.0 + 1e-9 {
        "growing"
    } else {
        "neutral"
    }
}

/// Human-facing report.
fn render_text(source: &str, states: &[&str], model: &DmdModel, dt: f64, pairs: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Koopman/DMD discovery from {source}");
    let _ = writeln!(out, "  states:    {}", states.join(", "));
    let _ = writeln!(out, "  snapshots: {pairs} pair(s), dt={}", format_number(dt));
    let _ = writeln!(out, "  rank:      {} of {}", model.rank(), states.len());
    let singular: Vec<String> =
        model.singular_values().iter().map(|value| format_number(*value)).collect();
    let _ = writeln!(out, "  singular values: {}", singular.join(", "));
    out.push('\n');

    let discrete = model.eigenvalues();
    let continuous = model.continuous_eigenvalues(dt);
    let _ = writeln!(out, "Discrete-time eigenvalues (x' ≈ A x, per step):");
    for value in discrete {
        let modulus = value.abs();
        let _ = writeln!(
            out,
            "  λ = {}   |λ|={}   ({})",
            render_complex(value),
            format_number(modulus),
            mode_behaviour(modulus)
        );
    }
    out.push('\n');
    let _ = writeln!(out, "Continuous-time eigenvalues (ln(λ)/dt = growth ± i·ω):");
    for value in &continuous {
        let _ = writeln!(
            out,
            "  μ = {}   (growth {}, angular freq {})",
            render_complex(value),
            format_number(value.re),
            format_number(value.im.abs())
        );
    }
    out.push('\n');

    let radius = spectral_radius(discrete);
    let stable = radius < 1.0 - 1e-9;
    let _ = writeln!(
        out,
        "Stability: spectral radius ρ = max|λ| = {} (asymptotically stable: {})",
        format_number(radius),
        if stable { "yes" } else { "no" }
    );
    let _ = writeln!(
        out,
        "note: DMD fits a LINEAR operator x' ≈ A x; it is a linear/lifted-linear \
approximation of the dynamics, not a symbolic nonlinear law, and writes no \
.lsworld bundle."
    );
    out
}

/// Stable, machine-readable report.
fn render_json(source: &str, states: &[&str], model: &DmdModel, dt: f64) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"method\": \"koopman-dmd\",");
    let _ = writeln!(out, "  \"source\": {},", json_string(source));
    let names: Vec<String> = states.iter().map(|name| json_string(name)).collect();
    let _ = writeln!(out, "  \"states\": [{}],", names.join(", "));
    let _ = writeln!(out, "  \"rank\": {},", model.rank());
    let _ = writeln!(out, "  \"dt\": {dt:.17e},");

    let singular: Vec<String> =
        model.singular_values().iter().map(|value| format!("{value:.17e}")).collect();
    let _ = writeln!(out, "  \"singular_values\": [{}],", singular.join(", "));

    let discrete: Vec<String> = model
        .eigenvalues()
        .iter()
        .map(|value| {
            format!(
                "{{\"re\": {:.17e}, \"im\": {:.17e}, \"modulus\": {:.17e}}}",
                value.re,
                value.im,
                value.abs()
            )
        })
        .collect();
    let _ = writeln!(out, "  \"discrete_eigenvalues\": [{}],", discrete.join(", "));

    let continuous: Vec<String> = model
        .continuous_eigenvalues(dt)
        .iter()
        .map(|value| format!("{{\"re\": {:.17e}, \"im\": {:.17e}}}", value.re, value.im))
        .collect();
    let _ = writeln!(out, "  \"continuous_eigenvalues\": [{}],", continuous.join(", "));

    let radius = spectral_radius(model.eigenvalues());
    let _ = writeln!(out, "  \"spectral_radius\": {radius:.17e},");
    let _ = writeln!(out, "  \"stable\": {}", radius < 1.0 - 1e-9);
    let _ = writeln!(out, "}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_command() {
        let help = help();
        assert!(help.contains("--state"));
        assert!(help.contains("DMD"));
        assert!(help.contains("eigenvalue"));
    }

    #[test]
    fn requires_state() {
        let error = run(&["data.csv".to_owned()]).unwrap_err();
        assert!(error.contains("--state") || error.contains("koopman"), "error: {error}");
    }

    #[test]
    fn spectral_radius_is_the_max_modulus() {
        let values = [Complex::new(0.5, 0.0), Complex::new(0.0, 0.9), Complex::new(0.1, 0.1)];
        assert!((spectral_radius(&values) - 0.9).abs() < 1e-12);
    }

    #[test]
    fn mode_behaviour_classifies_by_modulus() {
        assert_eq!(mode_behaviour(0.5), "decaying");
        assert_eq!(mode_behaviour(1.5), "growing");
        assert_eq!(mode_behaviour(1.0), "neutral");
    }
}
