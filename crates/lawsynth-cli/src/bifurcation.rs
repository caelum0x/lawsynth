//! `lawsynth bifurcation` — parameter continuation and bifurcation detection.
//!
//! Loads a continuous world whose laws contain a free **parameter** symbol,
//! sweeps that parameter across a range, re-locates the fixed points at each
//! value, stitches them into branches, and reports where a Jacobian eigenvalue
//! crosses the imaginary axis (a fold-family or Hopf bifurcation). All of the
//! numerics come from [`lawsynth_bifurcation::continuation`], which itself reuses
//! the stability/Jacobian/eigensolver stack.
//!
//! # What the world must provide
//!
//! Continuation needs a *parameterized* field: the symbol named by `--parameter`
//! must actually appear in at least one law. Discovered worlds that inline their
//! coefficients as plain constants have no free parameter, so there is nothing to
//! sweep — the command says so plainly rather than reporting a vacuous result.
//! Every *other* declared parameter is pinned at its stored value, so only the
//! swept parameter is free.

use std::fmt::Write as _;

use lawsynth_bifurcation::{
    Bifurcation, BifurcationKind, ContinuationReport, StabilityConfig, Sweep, continuation,
};
use lawsynth_core::Identifier;
use lawsynth_expr::symbols;
use lawsynth_report::format_number;

use crate::analysis::{
    fields_with_free, json_string, parse_range, parse_search_box, parse_usize, render_complex,
};

/// The default number of parameter grid points (mirrors the engine default).
const DEFAULT_STEPS: usize = 21;

/// Help text for `lawsynth bifurcation`.
pub fn help() -> String {
    "lawsynth bifurcation WORLD.lsworld --parameter NAME --range MIN:MAX \
--box LOW:HIGH[,LOW:HIGH...] [--steps N] [--grid N] [--json]\n\n\
Sweeps a free parameter of the world's vector field across [MIN, MAX], re-locates \
the fixed points at each value inside the search box (one LOW:HIGH interval per \
state), tracks them into branches, and detects bifurcations where a Jacobian \
eigenvalue crosses the imaginary axis. A real eigenvalue through zero is reported \
generically as a fold (saddle-node / transcritical / pitchfork); a complex pair \
crossing is a Hopf. The named parameter must appear in at least one law: a world \
that inlines its coefficients as constants has no free parameter to sweep. Every \
other declared parameter is held at its stored value. --json emits a stable \
machine-readable report."
        .to_owned()
}

/// Runs the `bifurcation` command.
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

    let mut parameter = None;
    let mut range = None;
    let mut search_box = None;
    let mut steps = DEFAULT_STEPS;
    let mut grid = None;
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
            "--parameter" => {
                parameter = Some(Identifier::new(value).map_err(|error| error.to_string())?);
            }
            "--range" => range = Some(parse_range(value)?),
            "--box" => search_box = Some(parse_search_box(value)?),
            "--steps" => steps = parse_usize(value, "--steps")?,
            "--grid" => grid = Some(parse_usize(value, "--grid")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = lawsynth_bundle::read_world(bundle).map_err(|error| error.to_string())?;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    let parameter = parameter.ok_or_else(|| "--parameter NAME is required".to_owned())?;
    let (min, max) = range.ok_or_else(|| "--range MIN:MAX is required".to_owned())?;
    let search_box = search_box.ok_or_else(|| {
        format!("--box is required (one LOW:HIGH interval per state, {} state(s))", states.len())
    })?;

    // The swept parameter stays free; all other declared parameters are pinned.
    let fields = fields_with_free(&world, std::slice::from_ref(&parameter));

    // Be honest when the parameter has no effect on the field: continuation would
    // otherwise report a vacuous "no bifurcations" that hides the real cause.
    let appears = fields.iter().any(|(_, expression)| symbols(expression).contains(&parameter));
    if !appears {
        return Err(format!(
            "parameter '{}' does not appear in any of the world's laws; bifurcation \
continuation needs a parameterized field (a world that inlines coefficients as \
constants has no free parameter to sweep)",
            parameter.as_str()
        ));
    }

    let sweep = Sweep::new(min, max, steps);
    let mut config = StabilityConfig::new(search_box);
    if let Some(grid) = grid {
        config = config.with_grid_resolution(grid);
    }

    let report = continuation(&fields, &states, &parameter, &sweep, &config)
        .map_err(|error| error.to_string())?;

    if as_json {
        Ok(render_json(bundle, min, max, steps, &report))
    } else {
        Ok(render_text(bundle, min, max, steps, &report))
    }
}

/// A short kind label plus, for folds, the family it stands in for.
fn kind_label(kind: BifurcationKind) -> &'static str {
    match kind {
        BifurcationKind::Fold => "fold (saddle-node / transcritical / pitchfork)",
        BifurcationKind::Hopf => "hopf (oscillation birth/death)",
    }
}

/// The stable JSON token for a kind.
fn kind_token(kind: BifurcationKind) -> &'static str {
    match kind {
        BifurcationKind::Fold => "fold",
        BifurcationKind::Hopf => "hopf",
    }
}

/// Human-facing report.
fn render_text(
    bundle: &str,
    min: f64,
    max: f64,
    steps: usize,
    report: &ContinuationReport,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Bifurcation continuation of {bundle}");
    let states: Vec<&str> = report.states.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states:    {}", states.join(", "));
    let _ = writeln!(
        out,
        "  parameter: {} over [{}, {}], {} grid point(s)",
        report.parameter.as_str(),
        format_number(min),
        format_number(max),
        steps
    );
    let _ = writeln!(out, "  branches:  {}", report.branch_count());
    out.push('\n');

    if report.bifurcations.is_empty() {
        let _ = writeln!(
            out,
            "No bifurcations detected across this range. The fixed points keep their \
stability throughout \u{2014} widen --range or refine --steps to search further."
        );
        return out;
    }

    let _ = writeln!(out, "Detected bifurcation(s): {}", report.bifurcation_count());
    for (number, bifurcation) in report.bifurcations.iter().enumerate() {
        let _ = writeln!(
            out,
            "  #{}  {}* = {}",
            number + 1,
            report.parameter.as_str(),
            format_number(bifurcation.parameter_value)
        );
        let _ = writeln!(out, "      kind:       {}", kind_label(bifurcation.kind));
        let _ = writeln!(out, "      branch:     {}", bifurcation.branch_id);
        let coordinates =
            crate::analysis::render_coordinates(&report.states, &bifurcation.fixed_point);
        let _ = writeln!(out, "      at:         {coordinates}");
        let _ = writeln!(out, "      eigenvalue: {}", render_complex(&bifurcation.eigenvalue));
    }
    out
}

/// Stable, machine-readable report.
fn render_json(
    bundle: &str,
    min: f64,
    max: f64,
    steps: usize,
    report: &ContinuationReport,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let states: Vec<String> =
        report.states.iter().map(|state| json_string(state.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", states.join(", "));
    let _ = writeln!(out, "  \"parameter\": {},", json_string(report.parameter.as_str()));
    let _ = writeln!(out, "  \"range\": {{\"min\": {min:.17e}, \"max\": {max:.17e}}},");
    let _ = writeln!(out, "  \"steps\": {steps},");
    let _ = writeln!(out, "  \"branch_count\": {},", report.branch_count());
    let _ = writeln!(out, "  \"bifurcations\": [");
    for (number, bifurcation) in report.bifurcations.iter().enumerate() {
        write_bifurcation_json(&mut out, bifurcation);
        let terminator = if number + 1 == report.bifurcations.len() { "    }" } else { "    }," };
        let _ = writeln!(out, "{terminator}");
    }
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

/// Writes the body (without the closing brace) of one bifurcation JSON object.
fn write_bifurcation_json(out: &mut String, bifurcation: &Bifurcation) {
    let coordinates: Vec<String> =
        bifurcation.fixed_point.iter().map(|value| format!("{value:.17e}")).collect();
    let _ = writeln!(out, "    {{");
    let _ = writeln!(out, "      \"parameter_value\": {:.17e},", bifurcation.parameter_value);
    let _ = writeln!(out, "      \"kind\": {},", json_string(kind_token(bifurcation.kind)));
    let _ = writeln!(out, "      \"branch_id\": {},", bifurcation.branch_id);
    let _ = writeln!(out, "      \"fixed_point\": [{}],", coordinates.join(", "));
    let _ = writeln!(
        out,
        "      \"eigenvalue\": {{\"re\": {:.17e}, \"im\": {:.17e}}}",
        bifurcation.eigenvalue.re, bifurcation.eigenvalue.im
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_required_flags() {
        let help = help();
        assert!(help.contains("--parameter"));
        assert!(help.contains("--range"));
        assert!(help.contains("--box"));
    }

    #[test]
    fn kind_labels_are_distinct() {
        assert_ne!(kind_token(BifurcationKind::Fold), kind_token(BifurcationKind::Hopf));
        assert!(kind_label(BifurcationKind::Hopf).contains("hopf"));
    }
}
