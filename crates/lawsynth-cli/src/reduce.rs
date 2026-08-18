//! `lawsynth reduce` — balanced-truncation model-order reduction.
//!
//! A discovered world gives a nonlinear field `ẋ = f(x)`. This command linearizes
//! it at a located fixed point to get `A = ∂f/∂x`, forms input/output maps, and
//! reduces the resulting linear system `(A, B, C)` to a lower order by balanced
//! truncation. The numerics come from [`lawsynth_modelreduce`].
//!
//! # How `A`, `B`, and `C` come from a world (honest assumptions)
//!
//! - `A` is the analytic Jacobian of the world's field at the **first** fixed
//!   point located inside `--box`. Reduction is of that *linearization*; balanced
//!   truncation additionally requires `A` to be Hurwitz (a *stable* fixed point),
//!   so an unstable equilibrium is rejected with a clear message.
//! - `B` defaults to the identity `I` (every state directly actuated) and `C`
//!   defaults to the identity `I` (every state measured), unless `--measure`
//!   selects a subset of measured states for `C`. These defaults are stated in
//!   the output; a different actuation/measurement structure would need a world
//!   that carries it explicitly.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_modelreduce::{Matrix, ReducedModel, ReductionSpec, balanced_truncation};
use lawsynth_report::format_number;

use crate::analysis::{
    classification_label, json_string, linearize_first, matrix_json, parse_identifiers,
    parse_number, parse_search_box, parse_usize, render_coordinates,
};

/// Help text for `lawsynth reduce`.
pub fn help() -> String {
    "lawsynth reduce WORLD.lsworld --box LOW:HIGH[,LOW:HIGH...] \
(--order K | --tolerance T) [--measure NAME[,NAME...]] [--grid N] [--json]\n\n\
Linearizes the world's field at the first fixed point located inside the search \
box (A = df/dx there) and reduces the linear system by balanced truncation. \
Choose the reduced order with --order K (keep K states) or --tolerance T (keep \
the fewest states whose discarded Hankel-singular-value energy is at most fraction \
T, 0 <= T < 1). B defaults to the identity (every state actuated); C defaults to \
the identity unless --measure selects the measured states. Balanced truncation \
requires a stable (Hurwitz) fixed point. Reports the Hankel singular values, the \
reduced order, and the H-infinity error bound. --json also emits the reduced \
matrices."
        .to_owned()
}

/// Runs the `reduce` command.
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
    let mut order = None;
    let mut tolerance = None;
    let mut measured = None;
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
            "--box" => search_box = Some(parse_search_box(value)?),
            "--order" => order = Some(parse_usize(value, "--order")?),
            "--tolerance" => tolerance = Some(parse_number(value, "--tolerance")?),
            "--measure" => measured = Some(parse_identifiers(value)?),
            "--grid" => grid = Some(parse_usize(value, "--grid")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = lawsynth_bundle::read_world(bundle).map_err(|error| error.to_string())?;
    let search_box =
        search_box.ok_or_else(|| "--box is required to locate a fixed point".to_owned())?;
    let spec = match (order, tolerance) {
        (Some(_), Some(_)) => {
            return Err("choose either --order K or --tolerance T, not both".to_owned());
        }
        (Some(k), None) => ReductionSpec::Order(k),
        (None, Some(t)) => ReductionSpec::EnergyTolerance(t),
        (None, None) => {
            return Err("one of --order K or --tolerance T is required".to_owned());
        }
    };

    let linear = linearize_first(&world, search_box, grid)?;
    let n = linear.states.len();
    let input_map = Matrix::identity(n);
    let output_map = match &measured {
        Some(measured) => build_output_map(&linear.states, measured)?,
        None => Matrix::identity(n),
    };

    let model = balanced_truncation(&linear.a, &input_map, &output_map, &spec)
        .map_err(|error| error.to_string())?;

    if as_json {
        Ok(render_json(bundle, &linear, measured.as_deref(), &model))
    } else {
        Ok(render_text(bundle, &linear, measured.as_deref(), &model))
    }
}

/// Builds the output map `C` selecting the measured states, in the named order.
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

/// A short description of the chosen `C`.
fn output_description(measured: Option<&[Identifier]>) -> String {
    match measured {
        Some(measured) => {
            let names: Vec<&str> = measured.iter().map(Identifier::as_str).collect();
            format!("C selects {}", names.join(", "))
        }
        None => "C = I (every state measured)".to_owned(),
    }
}

/// Human-facing report.
fn render_text(
    bundle: &str,
    linear: &crate::analysis::Linearization,
    measured: Option<&[Identifier]>,
    model: &ReducedModel,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Balanced-truncation model reduction of {bundle}");
    let states: Vec<&str> = linear.states.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states:      {} ({})", states.join(", "), linear.states.len());
    let _ = writeln!(
        out,
        "  fixed point: ({}) [{}]  ({} located, using the first)",
        render_coordinates(&linear.states, &linear.coordinates),
        classification_label(linear.classification),
        linear.points_found
    );
    let _ = writeln!(
        out,
        "  maps:        B = I (every state actuated), {}",
        output_description(measured)
    );
    out.push('\n');

    let _ = writeln!(out, "Hankel singular values (non-increasing):");
    for (index, sigma) in model.hankel_singular_values.iter().enumerate() {
        let _ = writeln!(out, "  sigma_{} = {}", index + 1, format_number(*sigma));
    }
    out.push('\n');

    let _ = writeln!(out, "Reduced order: {} of {}", model.order, linear.states.len());
    let _ = writeln!(
        out,
        "H-infinity error bound: |G - Gr|_inf <= {}",
        format_number(model.error_bound())
    );
    out.push('\n');
    let _ = writeln!(
        out,
        "note: reduction is of the linearization at the located fixed point; B = I and \
C = I unless --measure is given; balanced truncation requires a stable (Hurwitz) \
fixed point."
    );
    out
}

/// Stable, machine-readable report including the reduced matrices.
fn render_json(
    bundle: &str,
    linear: &crate::analysis::Linearization,
    measured: Option<&[Identifier]>,
    model: &ReducedModel,
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
    let measured_json = match measured {
        Some(measured) => {
            let names: Vec<String> =
                measured.iter().map(|name| json_string(name.as_str())).collect();
            format!("[{}]", names.join(", "))
        }
        None => "null".to_owned(),
    };
    let _ = writeln!(out, "  \"measured\": {measured_json},");
    let sigma: Vec<String> =
        model.hankel_singular_values.iter().map(|value| format!("{value:.17e}")).collect();
    let _ = writeln!(out, "  \"hankel_singular_values\": [{}],", sigma.join(", "));
    let _ = writeln!(out, "  \"order\": {},", model.order);
    let _ = writeln!(out, "  \"error_bound\": {:.17e},", model.error_bound());
    let _ = writeln!(out, "  \"reduced\": {{");
    let _ = writeln!(out, "    \"a\": {},", matrix_json(&model.a));
    let _ = writeln!(out, "    \"b\": {},", matrix_json(&model.b));
    let _ = writeln!(out, "    \"c\": {}", matrix_json(&model.c));
    let _ = writeln!(out, "  }}");
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
        assert!(help.contains("--order"));
        assert!(help.contains("--tolerance"));
        assert!(help.contains("Hankel"));
    }

    #[test]
    fn output_description_reports_defaults_and_selection() {
        assert!(output_description(None).contains("C = I"));
        assert!(output_description(Some(&[id("x")])).contains("selects x"));
    }
}
