//! `lawsynth invariants` — conserved-quantity (invariant) detection on a world.
//!
//! Loads a discovered/authored continuous world, reads its laws as an autonomous
//! vector field `ẋ = f(x)` (pinning every declared parameter at its stored value,
//! exactly as `stability`/`bifurcation` do), and runs the deterministic
//! [`lawsynth_invariants::detect_invariants`] engine over a candidate library of
//! monomials (optionally with `sin`/`cos` terms). Each detected conserved quantity
//! `H(x)` is printed as a human-readable combination of basis terms, e.g.
//! `1.00·x^2 + 1.00·y^2`, alongside its residual `‖L_f H‖` and singular value.
//!
//! A detection is a *hypothesis*, not a proof: the search only finds invariants
//! expressible in the chosen degree-`D` library, so an empty result is reported
//! honestly ("no conserved quantity expressible in the degree-D library") rather
//! than as a claim that the system conserves nothing.

use std::fmt::Write as _;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_invariants::{InvariantConfig, InvariantReport, detect_invariants};

use crate::analysis::{autonomous_fields, json_string, parse_number, parse_usize};

/// Coefficients whose rescaled magnitude is below this are dropped from the
/// printed combination (they are numerical dust, not real structure).
const DISPLAY_EPSILON: f64 = 1e-6;

/// Help text for `lawsynth invariants`.
pub fn help() -> String {
    "lawsynth invariants WORLD.lsworld [--degree D] [--trig] [--box LO:HI] \
[--resolution N] [--tolerance T] [--json]\n\n\
Searches for conserved quantities H(x) of the world's autonomous vector field: \
nonconstant functions whose Lie derivative L_f H = ∇H·f vanishes along the flow. \
H is parametrized over a candidate library of monomials up to total degree D \
(add --trig for sin/cos terms), sampled on a deterministic grid inside the box \
[LO, HI]^n, and each near-null direction of the Lie-derivative matrix is reported \
as an invariant with its residual and singular value. Every declared parameter is \
pinned at its stored value so the field is autonomous. The library bounds the \
search: an empty result means no conserved quantity expressible in the degree-D \
library was found. --json emits a stable machine-readable report.\n\n\
Defaults: --degree 2, --box -1:1.5, --resolution 5, --tolerance 1e-9."
        .to_owned()
}

/// Runs the `invariants` command.
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

    let mut config = InvariantConfig::default();
    let mut as_json = false;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--trig" {
            config.include_trigonometric = true;
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
            "--degree" => config.degree = parse_usize(value, "--degree")?,
            "--resolution" => config.resolution = parse_usize(value, "--resolution")?,
            "--tolerance" => config.tolerance = parse_tolerance(value)?,
            "--box" => {
                let (lo, hi) = parse_sample_box(value)?;
                config.sample_lo = lo;
                config.sample_hi = hi;
            }
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    let fields = autonomous_fields(&world);

    let report = detect_invariants(&fields, &states, &config).map_err(|error| error.to_string())?;

    if as_json {
        Ok(render_json(bundle, &config, &report))
    } else {
        Ok(render_text(bundle, &config, &report))
    }
}

/// Renders one invariant's coefficient vector as a signed combination of basis
/// terms, e.g. `1.00·x^2 + 1.00·y^2`. Coefficients are rescaled so the
/// largest-magnitude term reads as `1.00`, which turns the unit-norm vector the
/// engine returns into a natural, readable law. Terms below [`DISPLAY_EPSILON`]
/// after rescaling are dropped as numerical dust.
fn render_combination(labels: &[String], coefficients: &[f64]) -> String {
    let scale = coefficients.iter().fold(0.0_f64, |peak, value| peak.max(value.abs()));
    let scale = if scale > 0.0 { scale } else { 1.0 };

    let mut out = String::new();
    let mut first = true;
    for (label, coefficient) in labels.iter().zip(coefficients) {
        let scaled = coefficient / scale;
        if scaled.abs() < DISPLAY_EPSILON {
            continue;
        }
        let magnitude = scaled.abs();
        if first {
            if scaled < 0.0 {
                out.push('-');
            }
            let _ = write!(out, "{magnitude:.2}\u{b7}{label}");
            first = false;
        } else {
            let sign = if scaled < 0.0 { " - " } else { " + " };
            let _ = write!(out, "{sign}{magnitude:.2}\u{b7}{label}");
        }
    }
    // A canonically-normalized nonzero vector always keeps at least one term, but
    // stay honest if every term somehow fell below the display threshold.
    if out.is_empty() { "0 (degenerate)".to_owned() } else { out }
}

/// A short description of the candidate library used, for the report header.
fn library_summary(config: &InvariantConfig) -> String {
    if config.include_trigonometric {
        format!("monomials up to degree {} plus sin/cos terms", config.degree)
    } else {
        format!("monomials up to degree {}", config.degree)
    }
}

/// Human-facing report.
fn render_text(bundle: &str, config: &InvariantConfig, report: &InvariantReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Invariant analysis of {bundle}");
    let _ = writeln!(
        out,
        "  library: {} ({} basis term(s))",
        library_summary(config),
        report.basis_labels.len()
    );
    let _ = writeln!(
        out,
        "  sample box: [{}, {}]^n at {} point(s)/axis, tolerance {:.1e}",
        config.sample_lo, config.sample_hi, config.resolution, config.tolerance
    );
    out.push('\n');

    if report.invariants.is_empty() {
        let _ = writeln!(
            out,
            "No conserved quantity expressible in the degree-{} library was found within \
tolerance. This does not prove none exists \u{2014} raise --degree, add --trig, or \
widen --box to search a richer library.",
            config.degree
        );
        return out;
    }

    let _ = writeln!(out, "Conserved quantity(ies): {}", report.invariants.len());
    for (number, invariant) in report.invariants.iter().enumerate() {
        let combination = render_combination(&report.basis_labels, &invariant.coefficients);
        let _ = writeln!(out, "  #{}  H = {combination}", number + 1);
        let _ = writeln!(out, "      residual:       {:.6e}", invariant.residual);
        let _ = writeln!(out, "      singular value: {:.6e}", invariant.singular_value);
    }
    out
}

/// Stable, machine-readable report. Floats use the full 17-digit form so the JSON
/// is a faithful, deterministic image of the report.
fn render_json(bundle: &str, config: &InvariantConfig, report: &InvariantReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let _ = writeln!(out, "  \"degree\": {},", config.degree);
    let _ = writeln!(out, "  \"trigonometric\": {},", config.include_trigonometric);
    let _ = writeln!(
        out,
        "  \"sample_box\": {{\"lo\": {:.17e}, \"hi\": {:.17e}}},",
        config.sample_lo, config.sample_hi
    );
    let _ = writeln!(out, "  \"resolution\": {},", config.resolution);
    let _ = writeln!(out, "  \"tolerance\": {:.17e},", config.tolerance);
    let labels: Vec<String> = report.basis_labels.iter().map(|label| json_string(label)).collect();
    let _ = writeln!(out, "  \"basis_labels\": [{}],", labels.join(", "));
    let _ = writeln!(out, "  \"invariants\": [");
    for (number, invariant) in report.invariants.iter().enumerate() {
        let coefficients: Vec<String> =
            invariant.coefficients.iter().map(|value| format!("{value:.17e}")).collect();
        let _ = writeln!(out, "    {{");
        let _ = writeln!(
            out,
            "      \"combination\": {},",
            json_string(&render_combination(&report.basis_labels, &invariant.coefficients))
        );
        let _ = writeln!(out, "      \"coefficients\": [{}],", coefficients.join(", "));
        let _ = writeln!(out, "      \"residual\": {:.17e},", invariant.residual);
        let _ = writeln!(out, "      \"singular_value\": {:.17e}", invariant.singular_value);
        let terminator = if number + 1 == report.invariants.len() { "    }" } else { "    }," };
        let _ = writeln!(out, "{terminator}");
    }
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

/// Parses `LO:HI` into the shared per-axis sample interval.
fn parse_sample_box(value: &str) -> Result<(f64, f64), String> {
    let (low, high) =
        value.split_once(':').ok_or_else(|| format!("expected LO:HI in --box, got '{value}'"))?;
    let low = parse_number(low.trim(), "--box lower bound")?;
    let high = parse_number(high.trim(), "--box upper bound")?;
    if low >= high {
        return Err(format!("--box '{value}' must have lower bound below upper bound"));
    }
    Ok((low, high))
}

/// Parses a non-negative finite tolerance (zero admits only exact nullspace
/// directions).
fn parse_tolerance(value: &str) -> Result<f64, String> {
    let number = parse_number(value, "--tolerance")?;
    if number < 0.0 { Err("--tolerance must be >= 0".to_owned()) } else { Ok(number) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_flags() {
        let help = help();
        assert!(help.contains("--degree"));
        assert!(help.contains("--trig"));
        assert!(help.contains("--box"));
    }

    #[test]
    fn parses_sample_box_in_order() {
        assert_eq!(parse_sample_box("-1:1.5").unwrap(), (-1.0, 1.5));
        assert!(parse_sample_box("1:1").unwrap_err().contains("lower bound"));
    }

    #[test]
    fn renders_rescaled_energy_combination() {
        // A unit-norm x^2 + y^2 direction (1/√2 each) should read as 1.00 each.
        let labels = vec!["x^2".to_owned(), "x*y".to_owned(), "y^2".to_owned()];
        let weight = std::f64::consts::FRAC_1_SQRT_2;
        let coefficients = vec![weight, 0.0, weight];
        let combination = render_combination(&labels, &coefficients);
        assert_eq!(combination, "1.00\u{b7}x^2 + 1.00\u{b7}y^2");
    }

    #[test]
    fn drops_dust_terms_and_signs_negatives() {
        let labels = vec!["x".to_owned(), "y".to_owned(), "x^2".to_owned()];
        let coefficients = vec![2.0, -1.0, 1e-12];
        let combination = render_combination(&labels, &coefficients);
        assert_eq!(combination, "1.00\u{b7}x - 0.50\u{b7}y");
    }

    #[test]
    fn tolerance_rejects_negative() {
        assert!(parse_tolerance("-1").is_err());
        assert!(parse_tolerance("1e-9").is_ok());
    }
}
