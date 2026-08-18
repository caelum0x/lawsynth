//! `lawsynth discover --method weak-form` — noise-robust weak/integral-form
//! discovery (weak SINDy).
//!
//! Strong-form SINDy (the default `discover` path) fits `ẋ = Θ(x) Ξ` using
//! **estimated derivatives** of the data, and differentiating amplifies
//! observation noise. The weak form removes that step: it multiplies the ODE by
//! compactly-supported smooth test functions and integrates by parts, moving the
//! time-derivative off the noisy data and onto the analytic test functions. The
//! result is markedly more robust to noise. See
//! [`lawsynth_weakform::weak_discover`].
//!
//! # Why this renders coefficients rather than writing a `.lsworld`
//!
//! The weak-form engine returns per-state coefficient laws over a polynomial
//! candidate library ([`WeakResult`]); it does not assemble a serialized
//! [`lawsynth_world::World`] bundle the way strong-form `discover` does. Rather
//! than fabricate a world, this path honestly renders the recovered laws (and,
//! under `--json`, the full coefficient matrix) — the noise-robust analogue of
//! the strong-form summary.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn};
use lawsynth_report::format_number;
use lawsynth_weakform::{WeakConfig, WeakResult, weak_discover};

use crate::analysis::{json_string, parse_identifiers, parse_positive, parse_usize};
use crate::read_numeric_dataset;

/// Help text for `lawsynth discover --method weak-form`.
pub fn help() -> String {
    "lawsynth discover OBSERVATIONS.{csv,tsv,parquet} --method weak-form \
--time COLUMN --state NAME[,NAME...] [--degree D] [--threshold T] [--json]\n\n\
Discovers governing dynamics with the weak / integral form (weak SINDy): the ODE \
is multiplied by compactly-supported test functions and integrated by parts, so \
the observed data is NEVER differentiated. This is markedly more robust to \
observation noise than the strong-form default. Prints one recovered law per \
state over the polynomial candidate library. --degree sets the library degree, \
--threshold the sparsity cutoff. --json emits the full coefficient matrix. \
Unlike the strong-form path this does not write a .lsworld bundle: the weak-form \
engine returns coefficient laws, not a serialized world."
        .to_owned()
}

/// Runs the weak-form discovery path. `arguments` is the `discover` argument
/// slice with the `--method` flag already removed.
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
    let mut degree = None;
    let mut threshold = None;
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
            "--time" => time_column = Some(value.clone()),
            "--state" => states = Some(parse_identifiers(value)?),
            "--degree" => degree = Some(parse_usize(value, "--degree")?),
            "--threshold" => threshold = Some(parse_positive(value, "--threshold")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let time_column = time_column.ok_or_else(|| "--time COLUMN is required".to_owned())?;
    let states = states.ok_or_else(|| "--state NAME[,NAME...] is required".to_owned())?;

    let dataset = read_numeric_dataset(input, &time_column)?;
    let dataset = select_states(&dataset, &states)?;

    let mut config = WeakConfig::default();
    if let Some(degree) = degree {
        config.feature_degree = degree;
    }
    if let Some(threshold) = threshold {
        config.threshold = threshold;
    }

    let result = weak_discover(&dataset, &config).map_err(|error| error.to_string())?;

    if as_json { Ok(render_json(input, &result)) } else { Ok(render_text(input, &result)) }
}

/// Restricts the dataset to the requested state columns, preserving the time
/// axis. The weak-form engine treats every dataset column as a state, so this
/// makes `--state` a faithful selection just as strong-form `discover` does.
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

/// Human-facing report of the recovered weak-form laws.
fn render_text(source: &str, result: &WeakResult) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Weak-form discovery from {source}");
    let _ =
        writeln!(out, "  method:  weak / integral form (noise-robust; data is not differentiated)");
    let _ = writeln!(out, "  states:  {}", result.state_names.join(", "));
    let _ = writeln!(
        out,
        "  library: {} candidate term(s), {} test function(s)",
        result.diagnostics.library_terms, result.diagnostics.test_functions
    );
    let _ = writeln!(out, "  condition: {}", format_number(result.diagnostics.condition));
    out.push('\n');

    let _ = writeln!(out, "Discovered laws d/dt state = Σ coefficient · term:");
    for (law, residual) in result.laws.iter().zip(&result.diagnostics.residuals) {
        let _ = writeln!(out, "  {}", law.render());
        let _ = writeln!(
            out,
            "      active terms: {}, weak residual: {}",
            law.terms.len(),
            format_number(*residual)
        );
    }
    out.push('\n');
    let _ = writeln!(
        out,
        "note: weak/integral form avoids differentiating the observations, so it is \
more robust to noise than the strong-form default; it returns coefficient laws, \
not a .lsworld bundle."
    );
    out
}

/// Stable, machine-readable report: states, library terms, per-state laws and
/// the full coefficient matrix.
fn render_json(source: &str, result: &WeakResult) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"method\": \"weak-form\",");
    let _ = writeln!(out, "  \"source\": {},", json_string(source));
    let states: Vec<String> = result.state_names.iter().map(|name| json_string(name)).collect();
    let _ = writeln!(out, "  \"states\": [{}],", states.join(", "));
    let terms: Vec<String> = result.term_names.iter().map(|name| json_string(name)).collect();
    let _ = writeln!(out, "  \"library_terms\": [{}],", terms.join(", "));
    let _ = writeln!(out, "  \"test_functions\": {},", result.diagnostics.test_functions);
    let _ = writeln!(out, "  \"condition\": {:.17e},", result.diagnostics.condition);
    let _ = writeln!(out, "  \"laws\": [");
    for (number, law) in result.laws.iter().enumerate() {
        let residual = result.diagnostics.residuals.get(number).copied().unwrap_or(0.0);
        let rendered: Vec<String> = law
            .terms
            .iter()
            .map(|term| {
                format!(
                    "{{\"term\": {}, \"coefficient\": {:.17e}}}",
                    json_string(&term.name),
                    term.coefficient
                )
            })
            .collect();
        let _ = writeln!(out, "    {{");
        let _ = writeln!(out, "      \"state\": {},", json_string(&law.state));
        let _ = writeln!(out, "      \"residual\": {residual:.17e},");
        let _ = writeln!(out, "      \"terms\": [{}]", rendered.join(", "));
        let terminator = if number + 1 == result.laws.len() { "    }" } else { "    }," };
        let _ = writeln!(out, "{terminator}");
    }
    let _ = writeln!(out, "  ],");

    let coefficient_rows: Vec<String> = result
        .coefficients
        .iter()
        .map(|row| {
            let cells: Vec<String> = row.iter().map(|value| format!("{value:.17e}")).collect();
            format!("[{}]", cells.join(", "))
        })
        .collect();
    let _ = writeln!(out, "  \"coefficients\": [{}]", coefficient_rows.join(", "));
    let _ = writeln!(out, "}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_method() {
        let help = help();
        assert!(help.contains("weak-form"));
        assert!(help.contains("--state"));
        assert!(help.contains("not differentiated") || help.contains("NEVER differentiated"));
    }

    #[test]
    fn requires_time_and_state() {
        let error = run(&["data.csv".to_owned()]).unwrap_err();
        assert!(error.contains("--time") || error.contains("weak-form"), "error: {error}");
    }
}
