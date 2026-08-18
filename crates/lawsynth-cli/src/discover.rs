use std::fmt::Write;

use lawsynth_discovery::{DiscoveryConfig, StateCoefficientEnsemble};

/// Renders the stable one-line summary returned by successful discovery.
pub fn discovery_summary(config: &DiscoveryConfig, mse: f64, complexity: usize) -> String {
    format!(
        "discovered {} state laws: mse={mse:.6e}, complexity={complexity}\n",
        config.state.len()
    )
}

/// Renders the per-state bootstrap coefficient uncertainty as a stable text
/// block. Each retained candidate term is shown with its bootstrap-mean
/// coefficient, percentile confidence interval `[lower, upper]`, and inclusion
/// probability. Terms selected inconsistently across resamples appear honestly
/// with a low `incl=` rather than being hidden.
///
/// The header labels the intervals as **bootstrap percentile approximations**:
/// they carry no exact frequentist coverage guarantee, and a small `B` simply
/// widens them. Ordering is deterministic (states then library column order).
pub fn render_coefficient_uncertainty(
    ensembles: &[StateCoefficientEnsemble],
    resamples: usize,
    confidence: f64,
) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "coefficient uncertainty (bootstrap percentile, B={resamples}, confidence={confidence:.2}; approximate):"
    );
    for state in ensembles {
        let _ = writeln!(out, "  state {}:", state.state.as_str());
        for (name, term) in state.term_names.iter().zip(&state.ensemble.terms) {
            let _ = writeln!(
                out,
                "    {}: {:.4} [{:.4}, {:.4}] incl={:.2}",
                name, term.mean, term.lower, term.upper, term.inclusion_probability
            );
        }
    }
    out
}

/// Renders discovery as a stable machine-readable JSON report.
///
/// The `coefficient_uncertainty` object is `null` unless the opt-in bootstrap
/// ran; when present it carries the method label, resample count, confidence
/// level, and per-state, per-term intervals and inclusion probabilities. The
/// intervals are bootstrap approximations (`"approximate": true`).
pub fn discover_json(
    mse: f64,
    complexity: usize,
    solver: &str,
    states: usize,
    coefficient_uncertainty: Option<&[StateCoefficientEnsemble]>,
    resamples: usize,
    confidence: f64,
) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"mse\": {mse:.6e},");
    let _ = writeln!(out, "  \"complexity\": {complexity},");
    let _ = writeln!(out, "  \"solver\": {},", json_string(solver));
    let _ = writeln!(out, "  \"states\": {states},");
    match coefficient_uncertainty {
        None => out.push_str("  \"coefficient_uncertainty\": null\n"),
        Some(ensembles) => {
            out.push_str("  \"coefficient_uncertainty\": {\n");
            out.push_str("    \"method\": \"bootstrap-percentile\",\n");
            out.push_str("    \"approximate\": true,\n");
            let _ = writeln!(out, "    \"resamples\": {resamples},");
            let _ = writeln!(out, "    \"confidence\": {confidence:.6},");
            out.push_str("    \"states\": [\n");
            for (state_index, state) in ensembles.iter().enumerate() {
                out.push_str("      {\n");
                let _ = writeln!(out, "        \"state\": {},", json_string(state.state.as_str()));
                out.push_str("        \"terms\": [\n");
                let terms = state.term_names.iter().zip(&state.ensemble.terms);
                let count = state.term_names.len();
                for (term_index, (name, term)) in terms.enumerate() {
                    let _ = write!(
                        out,
                        "          {{\"term\": {}, \"coefficient\": {:.6e}, \"lower\": {:.6e}, \
\"upper\": {:.6e}, \"standard_error\": {:.6e}, \"inclusion_probability\": {:.6}}}",
                        json_string(name),
                        term.mean,
                        term.lower,
                        term.upper,
                        term.standard_error,
                        term.inclusion_probability
                    );
                    out.push_str(if term_index + 1 < count { ",\n" } else { "\n" });
                }
                out.push_str("        ]\n");
                out.push_str(if state_index + 1 < ensembles.len() {
                    "      },\n"
                } else {
                    "      }\n"
                });
            }
            out.push_str("    ]\n");
            out.push_str("  }\n");
        }
    }
    out.push_str("}\n");
    out
}

/// Escapes a string as a JSON string literal (control characters and quotes).
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            control if (control as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}
