//! `lawsynth sde` — stochastic (drift/diffusion) law discovery from a sample path.
//!
//! Given a noisy sample path of a diagonal-noise Itô SDE
//! `dX = a(X) dt + b(X) dW`, this command estimates the **drift** `a(x)` and the
//! **diffusion** `b²(x)` from the binned Kramers–Moyal conditional moments and
//! sparse-regresses each onto a polynomial candidate library. See
//! [`lawsynth_sde::discover_sde`].
//!
//! # Why a dedicated command
//!
//! The input is a single noisy time series (one sample path) and the output is a
//! pair of closed-form laws (drift and diffusion) plus a binned conditional-moment
//! table — a different contract from strong-form `discover`, which fits a
//! deterministic field and serializes a world. It gets its own command and prints
//! a method-appropriate summary; it writes no `.lsworld` bundle.

use std::fmt::Write as _;

use lawsynth_report::format_number;
use lawsynth_sde::{BinRule, DiscoveredLaw, SdeConfig, SdeModel, StateModel, discover_sde};

use crate::analysis::{json_string, parse_identifiers, parse_positive, parse_usize};
use crate::read_numeric_dataset;

/// The number of leading binned-table rows shown in the text summary.
const TABLE_PREVIEW_ROWS: usize = 6;

/// Help text for `lawsynth sde`.
pub fn help() -> String {
    "lawsynth sde OBSERVATIONS.{csv,tsv,parquet} --state NAME[,NAME...] \
[--time COLUMN] [--bins N] [--min-bin K] [--degree D] [--threshold T] [--json]\n\n\
Discovers a stochastic differential equation dX = a(X) dt + b(X) dW from a noisy \
sample path: the drift a(x) and diffusion b²(x) are estimated from binned \
Kramers–Moyal conditional moments and sparse-regressed onto a polynomial \
library. Prints the recovered drift and diffusion laws, a binned-table preview, \
and the trusted-bin count. --bins sets the state-space partition count, --min-bin \
the minimum occupancy a bin needs to be trusted, --degree the library degree, \
--threshold the sparsity cutoff. --json emits the laws and the binned table. \
This is a STATISTICAL estimator: accuracy grows with path length and rarely-\
visited bins are unreliable; it writes no .lsworld bundle."
        .to_owned()
}

/// Runs the `sde` command.
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
    let mut bins = None;
    let mut min_bin = None;
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
            "--state" => states = Some(parse_identifiers(value)?),
            "--time" => time_column = Some(value.clone()),
            "--bins" => bins = Some(parse_usize(value, "--bins")?),
            "--min-bin" => min_bin = Some(parse_usize(value, "--min-bin")?),
            "--degree" => degree = Some(parse_usize(value, "--degree")?),
            "--threshold" => threshold = Some(parse_positive(value, "--threshold")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let states = states.ok_or_else(|| "--state NAME[,NAME...] is required".to_owned())?;
    let time_column = time_column.unwrap_or_else(|| "time".to_owned());

    let dataset = read_numeric_dataset(input, &time_column)?;

    let mut config = SdeConfig::new().with_state_columns(states);
    if let Some(bins) = bins {
        config = config.with_bins(BinRule::Count(bins));
    }
    if let Some(min_bin) = min_bin {
        config = config.with_min_bin_count(min_bin);
    }
    if let Some(degree) = degree {
        config = config.with_polynomial_degree(degree);
    }
    if let Some(threshold) = threshold {
        config.sparse.threshold = threshold;
    }

    let model = discover_sde(&dataset, &config).map_err(|error| error.to_string())?;

    if as_json { Ok(render_json(input, &model)) } else { Ok(render_text(input, &model)) }
}

/// Renders a discovered polynomial law's active terms as `c*x^p + ...`.
fn render_law(law: &DiscoveredLaw) -> String {
    let mut active = law.active_terms().peekable();
    if active.peek().is_none() {
        return "0".to_owned();
    }
    active
        .map(|term| format!("{}*{}", format_number(term.coefficient), term.label))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Human-facing report.
fn render_text(source: &str, model: &SdeModel) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "SDE discovery from {source}");
    let _ =
        writeln!(out, "  increments: {} (dt={})", model.increment_count, format_number(model.dt));
    let _ = writeln!(out, "  bins:       {}", describe_bin_rule(model.bin_rule));
    out.push('\n');

    for state in &model.states {
        render_state_text(&mut out, state);
    }

    let _ = writeln!(
        out,
        "note: statistical Kramers–Moyal estimator — accuracy grows with path length, \
and rarely-visited (low-occupancy) bins are unreliable."
    );
    out
}

/// Appends one state's drift/diffusion laws and a binned-table preview.
fn render_state_text(out: &mut String, state: &StateModel) {
    let _ = writeln!(out, "State {}:", state.state.as_str());
    let _ = writeln!(out, "  drift     a(x)  = {}", render_law(&state.drift_law));
    let _ = writeln!(out, "  diffusion b²(x) = {}", render_law(&state.diffusion_law));
    let _ = writeln!(out, "  trusted bins: {} of {}", state.trusted_bins, state.bins.len());
    if !state.bins.is_empty() {
        let _ = writeln!(out, "  binned table (x_center: drift, diffusion, count):");
        for bin in state.bins.iter().take(TABLE_PREVIEW_ROWS) {
            let _ = writeln!(
                out,
                "    {}: {}, {} (n={})",
                format_number(bin.x_center),
                format_number(bin.drift),
                format_number(bin.diffusion),
                bin.count
            );
        }
        if state.bins.len() > TABLE_PREVIEW_ROWS {
            let _ = writeln!(out, "    … {} more bin(s)", state.bins.len() - TABLE_PREVIEW_ROWS);
        }
    }
    out.push('\n');
}

/// A stable label for the applied bin rule.
fn describe_bin_rule(rule: BinRule) -> String {
    match rule {
        BinRule::Count(count) => format!("count({count})"),
        BinRule::Width(width) => format!("width({})", format_number(width)),
    }
}

/// Stable, machine-readable report.
fn render_json(source: &str, model: &SdeModel) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"method\": \"sde-kramers-moyal\",");
    let _ = writeln!(out, "  \"source\": {},", json_string(source));
    let _ = writeln!(out, "  \"dt\": {:.17e},", model.dt);
    let _ = writeln!(out, "  \"increments\": {},", model.increment_count);
    let _ = writeln!(out, "  \"states\": [");
    for (number, state) in model.states.iter().enumerate() {
        let _ = writeln!(out, "    {{");
        let _ = writeln!(out, "      \"state\": {},", json_string(state.state.as_str()));
        let _ = writeln!(out, "      \"trusted_bins\": {},", state.trusted_bins);
        let _ = writeln!(out, "      \"drift\": {},", law_json(&state.drift_law));
        let _ = writeln!(out, "      \"diffusion\": {},", law_json(&state.diffusion_law));
        let bins: Vec<String> = state
            .bins
            .iter()
            .map(|bin| {
                format!(
                    "{{\"x_center\": {:.17e}, \"drift\": {:.17e}, \"diffusion\": {:.17e}, \
\"count\": {}}}",
                    bin.x_center, bin.drift, bin.diffusion, bin.count
                )
            })
            .collect();
        let _ = writeln!(out, "      \"bins\": [{}]", bins.join(", "));
        let terminator = if number + 1 == model.states.len() { "    }" } else { "    }," };
        let _ = writeln!(out, "{terminator}");
    }
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

/// A discovered law as a JSON object with its active terms and residual.
fn law_json(law: &DiscoveredLaw) -> String {
    let terms: Vec<String> = law
        .active_terms()
        .map(|term| {
            format!(
                "{{\"label\": {}, \"power\": {}, \"coefficient\": {:.17e}}}",
                json_string(&term.label),
                term.power,
                term.coefficient
            )
        })
        .collect();
    format!(
        "{{\"terms\": [{}], \"residual_sum_squares\": {:.17e}}}",
        terms.join(", "),
        law.residual_sum_squares
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_flags() {
        let help = help();
        assert!(help.contains("--state"));
        assert!(help.contains("drift"));
        assert!(help.contains("diffusion"));
        assert!(help.contains("STATISTICAL"));
    }

    #[test]
    fn requires_state() {
        let error = run(&["data.csv".to_owned()]).unwrap_err();
        assert!(error.contains("--state") || error.contains("sde"), "error: {error}");
    }

    #[test]
    fn describes_bin_rules() {
        assert_eq!(describe_bin_rule(BinRule::Count(24)), "count(24)");
        assert!(describe_bin_rule(BinRule::Width(0.5)).starts_with("width("));
    }
}
