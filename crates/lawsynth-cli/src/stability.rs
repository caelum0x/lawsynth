//! `lawsynth stability` — fixed-point and linear-stability analysis of a world.
//!
//! Loads a discovered/authored continuous world, reads its laws as an autonomous
//! vector field `ẋ = f(x)`, and runs the deterministic
//! [`lawsynth_stability::analyze_stability`] engine over a caller-provided search
//! box. Each located fixed point is printed with its coordinates, Jacobian
//! eigenvalues, and linear-stability classification.
//!
//! The search box is mandatory and load-bearing: it fixes both where the Newton
//! seeds start and which roots are reported (roots outside the box are dropped).
//! The command is honest about the search — it always reports how many seeds were
//! tried and how many converged, so an empty result reads as "the search found
//! nothing in this box" rather than "the system has no fixed points". Marginal /
//! center points are reported as inconclusive because the linearization cannot
//! decide them.

use std::fmt::Write as _;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_report::format_number;
use lawsynth_stability::{
    Classification, Complex, FixedPoint, StabilityConfig, StabilityReport, analyze_stability,
};
use lawsynth_world::World;

/// Help text for `lawsynth stability`.
pub fn help() -> String {
    "lawsynth stability WORLD.lsworld --box LOW:HIGH[,LOW:HIGH...] [--grid N] \
[--tolerance V] [--dedup V] [--marginal-band V] [--max-iterations N] \
[--divergence V] [--json]\n\n\
Locates the fixed points f(x)=0 of a world's autonomous vector field inside the \
given search box (one LOW:HIGH interval per state, in state order) and classifies \
each by its Jacobian eigenvalues (stable/unstable node or spiral, saddle, center, \
or marginal). The search box is required; roots outside it are dropped. The report \
states how many deterministic seeds converged, so an empty result means the search \
found nothing in this box. --json emits a stable machine-readable report."
        .to_owned()
}

/// Runs the `stability` command.
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
    let mut grid = None;
    let mut tolerance = None;
    let mut dedup = None;
    let mut marginal_band = None;
    let mut max_iterations = None;
    let mut divergence = None;
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
            "--grid" => grid = Some(parse_usize(value, "--grid")?),
            "--tolerance" => tolerance = Some(parse_positive(value, "--tolerance")?),
            "--dedup" => dedup = Some(parse_number(value, "--dedup")?),
            "--marginal-band" => marginal_band = Some(parse_number(value, "--marginal-band")?),
            "--max-iterations" => max_iterations = Some(parse_usize(value, "--max-iterations")?),
            "--divergence" => divergence = Some(parse_positive(value, "--divergence")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    let fields = fields_of(&world);

    let search_box = search_box.ok_or_else(|| {
        format!("--box is required (one LOW:HIGH interval per state, {} state(s))", states.len())
    })?;
    let mut config = StabilityConfig::new(search_box);
    if let Some(grid) = grid {
        config = config.with_grid_resolution(grid);
    }
    if let Some(tolerance) = tolerance {
        config = config.with_tolerance(tolerance);
    }
    if let Some(dedup) = dedup {
        config = config.with_dedup_tolerance(dedup);
    }
    if let Some(band) = marginal_band {
        config = config.with_marginal_band(band);
    }
    if let Some(iterations) = max_iterations {
        config = config.with_max_iterations(iterations);
    }
    if let Some(divergence) = divergence {
        config = config.with_divergence_limit(divergence);
    }

    let report = analyze_stability(&fields, &states, &config).map_err(|error| error.to_string())?;
    if as_json { Ok(render_json(bundle, &report)) } else { Ok(render_text(bundle, &report)) }
}

/// Reads a world's laws as `(state, right-hand side)` field pairs.
fn fields_of(world: &World) -> Vec<(Identifier, Expr)> {
    world.laws().iter().map(|(target, law)| (target.clone(), law.expression.clone())).collect()
}

/// A human-readable name for a classification verdict.
fn classification_label(classification: Classification) -> &'static str {
    match classification {
        Classification::StableNode => "stable node",
        Classification::StableSpiral => "stable spiral",
        Classification::UnstableNode => "unstable node",
        Classification::UnstableSpiral => "unstable spiral",
        Classification::Saddle => "saddle",
        Classification::Center => "center (marginal, inconclusive)",
        Classification::Marginal => "marginal (inconclusive)",
    }
}

/// Renders a single complex eigenvalue as `a + b i` with signed imaginary part.
fn render_eigenvalue(eigenvalue: &Complex) -> String {
    let sign = if eigenvalue.im < 0.0 { '-' } else { '+' };
    format!("{} {sign} {}i", format_number(eigenvalue.re), format_number(eigenvalue.im.abs()))
}

/// Renders a coordinate vector against the state order, e.g. `x=0, y=1`.
fn render_coordinates(states: &[Identifier], point: &FixedPoint) -> String {
    states
        .iter()
        .zip(&point.coordinates)
        .map(|(state, value)| format!("{}={}", state.as_str(), format_number(*value)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Human-facing report.
pub(crate) fn render_text(bundle: &str, report: &StabilityReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Stability analysis of {bundle}");
    let states: Vec<&str> = report.states.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  states: {}", states.join(", "));
    let _ = writeln!(
        out,
        "  seeds: {} tried, {} converged",
        report.seeds_total, report.seeds_converged
    );
    out.push('\n');

    if report.fixed_points.is_empty() {
        let _ = writeln!(
            out,
            "No fixed points found inside the search box. This does not prove none \
exist \u{2014} widen the box or refine the grid to search elsewhere."
        );
        return out;
    }

    let _ = writeln!(out, "Fixed point(s): {}", report.fixed_points.len());
    for (number, point) in report.fixed_points.iter().enumerate() {
        let _ = writeln!(out, "  #{}  ({})", number + 1, render_coordinates(&report.states, point));
        let _ =
            writeln!(out, "      classification: {}", classification_label(point.classification));
        let eigenvalues: Vec<String> = point.eigenvalues.iter().map(render_eigenvalue).collect();
        let _ = writeln!(out, "      eigenvalues:    {}", eigenvalues.join(", "));
        if point.classification.is_inconclusive() {
            let _ = writeln!(
                out,
                "      note: non-hyperbolic \u{2014} linear stability is inconclusive here."
            );
        }
    }
    out
}

/// Stable, machine-readable report. Floats use the full 17-digit form so the JSON
/// is a faithful, deterministic image of the report.
pub(crate) fn render_json(bundle: &str, report: &StabilityReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"world\": {},", json_string(bundle));
    let states: Vec<String> =
        report.states.iter().map(|state| json_string(state.as_str())).collect();
    let _ = writeln!(out, "  \"states\": [{}],", states.join(", "));
    let _ = writeln!(out, "  \"seeds_total\": {},", report.seeds_total);
    let _ = writeln!(out, "  \"seeds_converged\": {},", report.seeds_converged);
    let _ = writeln!(out, "  \"fixed_points\": [");
    for (number, point) in report.fixed_points.iter().enumerate() {
        let coordinates: Vec<String> =
            point.coordinates.iter().map(|value| format!("{value:.17e}")).collect();
        let eigenvalues: Vec<String> = point
            .eigenvalues
            .iter()
            .map(|eigenvalue| {
                format!("{{\"re\": {:.17e}, \"im\": {:.17e}}}", eigenvalue.re, eigenvalue.im)
            })
            .collect();
        let _ = writeln!(out, "    {{");
        let _ = writeln!(out, "      \"coordinates\": [{}],", coordinates.join(", "));
        let _ = writeln!(
            out,
            "      \"classification\": {},",
            json_string(classification_label(point.classification))
        );
        let _ =
            writeln!(out, "      \"inconclusive\": {},", point.classification.is_inconclusive());
        let _ = writeln!(out, "      \"eigenvalues\": [{}]", eigenvalues.join(", "));
        let terminator = if number + 1 == report.fixed_points.len() { "    }" } else { "    }," };
        let _ = writeln!(out, "{terminator}");
    }
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

/// Minimal JSON string escaping (mirrors the convention in `compare.rs`).
fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Parses `LOW:HIGH[,LOW:HIGH...]` into per-state search intervals.
fn parse_search_box(value: &str) -> Result<Vec<(f64, f64)>, String> {
    let mut intervals = Vec::new();
    for entry in value.split(',') {
        let entry = entry.trim();
        let (low, high) = entry
            .split_once(':')
            .ok_or_else(|| format!("expected LOW:HIGH in --box, got '{entry}'"))?;
        let low = parse_number(low.trim(), "--box lower bound")?;
        let high = parse_number(high.trim(), "--box upper bound")?;
        if low > high {
            return Err(format!("--box interval '{entry}' has lower bound above upper bound"));
        }
        intervals.push((low, high));
    }
    if intervals.is_empty() {
        return Err("expected at least one LOW:HIGH interval in --box".to_owned());
    }
    Ok(intervals)
}

fn parse_number(value: &str, flag: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}' for {flag}"))?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(format!("value '{value}' for {flag} must be finite"))
    }
}

fn parse_positive(value: &str, flag: &str) -> Result<f64, String> {
    let number = parse_number(value, flag)?;
    if number > 0.0 { Ok(number) } else { Err(format!("value for {flag} must be > 0")) }
}

fn parse_usize(value: &str, flag: &str) -> Result<usize, String> {
    value.parse().map_err(|_| format!("invalid count '{value}' for {flag}"))
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::{Expr, UnaryOperator};
    use lawsynth_world::{ContinuousLaw, Variable, VariableRole};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    /// A linear stable node at the origin: x' = -x, y' = -2y.
    fn stable_node_world() -> World {
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            [],
            [
                ContinuousLaw::new(
                    id("x"),
                    Expr::unary(UnaryOperator::Negate, Expr::symbol(id("x"))),
                ),
                ContinuousLaw::new(
                    id("y"),
                    Expr::product(Expr::constant(-2.0), Expr::symbol(id("y"))),
                ),
            ],
        )
        .unwrap()
    }

    #[test]
    fn parses_multi_state_search_box() {
        let intervals = parse_search_box("-1:1, -2:2").unwrap();
        assert_eq!(intervals, vec![(-1.0, 1.0), (-2.0, 2.0)]);
    }

    #[test]
    fn rejects_inverted_interval() {
        assert!(parse_search_box("1:-1").unwrap_err().contains("lower bound"));
    }

    #[test]
    fn help_documents_the_required_box() {
        let help = help();
        assert!(help.contains("--box"));
        assert!(help.contains("required"));
    }

    #[test]
    fn classifies_origin_as_stable_node_from_fields() {
        let world = stable_node_world();
        let fields = fields_of(&world);
        let states: Vec<Identifier> = world.state_ids().cloned().collect();
        let config = StabilityConfig::new(vec![(-1.0, 1.0), (-1.0, 1.0)]);
        let report = analyze_stability(&fields, &states, &config).unwrap();
        assert_eq!(report.fixed_points.len(), 1);
        assert_eq!(report.fixed_points[0].classification, Classification::StableNode);
        let text = render_text("mem", &report);
        assert!(text.contains("stable node"));
        assert!(text.contains("x=0"));
    }

    #[test]
    fn json_report_lists_the_fixed_point() {
        let world = stable_node_world();
        let fields = fields_of(&world);
        let states: Vec<Identifier> = world.state_ids().cloned().collect();
        let config = StabilityConfig::new(vec![(-1.0, 1.0), (-1.0, 1.0)]);
        let report = analyze_stability(&fields, &states, &config).unwrap();
        let json = render_json("mem", &report);
        assert!(json.contains("\"classification\": \"stable node\""));
        assert!(json.contains("\"seeds_converged\""));
    }
}
