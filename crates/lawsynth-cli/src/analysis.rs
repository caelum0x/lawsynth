//! Shared helpers for the analysis-layer subcommands (`bifurcation`,
//! `sensitivity`, `estimate`, `reduce`).
//!
//! These four commands all load a world, turn its laws into `(state, rhs)`
//! expression fields, and feed the result to an engine crate. Two things recur
//! across them and live here so each command stays small and honest:
//!
//! - **Argument parsing** (`--box`, numbers, counts, identifier lists) and the
//!   minimal JSON-string escaping, mirroring the convention already used by
//!   `stability.rs` and `control.rs`.
//! - **Linearization at a fixed point** — the `estimate` and `reduce` commands
//!   both need the state matrix `A = ∂f/∂x` evaluated at a located equilibrium.
//!   [`linearize_first`] locates the fixed points inside a search box with the
//!   real [`lawsynth_stability`] engine, then builds `A` from the analytic
//!   Jacobian of [`lawsynth_jacobian`]. It is deliberately honest about the
//!   search: it returns how many equilibria were found and which one was used.

use lawsynth_bifurcation::substitute;
use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, Expr};
use lawsynth_jacobian::analytic_jacobian;
use lawsynth_koopman::{Complex, Matrix};
use lawsynth_report::format_number;
use lawsynth_stability::{Classification, StabilityConfig, analyze_stability};
use lawsynth_world::World;

/// A world's laws with **every** declared parameter replaced by its constant
/// value, yielding a parameter-free autonomous field `ẋ = f(x)`.
///
/// Fixed-point location and Jacobian evaluation both require an autonomous field
/// (a free parameter would have no value to evaluate at). Discovered worlds that
/// already inline their coefficients as constants have no parameters, so this is
/// a no-op for them; authored/parameterized worlds are pinned at their declared
/// parameter values.
pub fn autonomous_fields(world: &World) -> Vec<(Identifier, Expr)> {
    fields_with_free(world, &[])
}

/// A world's laws with every declared parameter **except** those in `free`
/// substituted by its constant value.
///
/// The `free` symbols are left in the expressions so an engine (continuation,
/// forward sensitivity) can sweep or differentiate with respect to them, while
/// all other parameters are pinned at their declared values so no spurious free
/// symbol survives.
pub fn fields_with_free(world: &World, free: &[Identifier]) -> Vec<(Identifier, Expr)> {
    world
        .laws()
        .iter()
        .map(|(target, law)| {
            let mut expression = law.expression.clone();
            for (name, parameter) in world.parameters() {
                if !free.contains(name) {
                    expression = substitute(&expression, name, parameter.value);
                }
            }
            (target.clone(), expression)
        })
        .collect()
}

/// The linearization of a world's field at a located fixed point.
pub struct Linearization {
    /// The state ordering that indexes coordinates and the matrix `A`.
    pub states: Vec<Identifier>,
    /// The fixed-point coordinates, in `states` order.
    pub coordinates: Vec<f64>,
    /// The linear-stability verdict at the fixed point.
    pub classification: Classification,
    /// The state matrix `A = ∂f/∂x` evaluated at the fixed point.
    pub a: Matrix,
    /// How many fixed points the search located inside the box.
    pub points_found: usize,
}

/// Locates the fixed points of a world's autonomous field inside `search_box`
/// and linearizes at the first one, returning `A = ∂f/∂x` there.
///
/// The choice of the *first* fixed point (in the stability engine's
/// deterministic order) is arbitrary but reproducible; the returned
/// [`Linearization::points_found`] lets the caller report the ambiguity honestly.
pub fn linearize_first(
    world: &World,
    search_box: Vec<(f64, f64)>,
    grid: Option<usize>,
) -> Result<Linearization, String> {
    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    let fields = autonomous_fields(world);

    let mut config = StabilityConfig::new(search_box);
    if let Some(grid) = grid {
        config = config.with_grid_resolution(grid);
    }
    let report = analyze_stability(&fields, &states, &config).map_err(|error| error.to_string())?;

    let point = report.fixed_points.first().ok_or_else(|| {
        format!(
            "no fixed point found inside the search box ({} of {} seeds converged); \
widen --box or refine --grid",
            report.seeds_converged, report.seeds_total
        )
    })?;

    let environment: Environment =
        states.iter().cloned().zip(point.coordinates.iter().copied()).collect();
    let jacobian = analytic_jacobian(&fields, &states).map_err(|error| error.to_string())?;
    let dense = jacobian.evaluate(&environment).map_err(|error| error.to_string())?;
    let a = Matrix::from_rows(dense).map_err(|error| error.to_string())?;

    Ok(Linearization {
        states,
        coordinates: point.coordinates.clone(),
        classification: point.classification,
        a,
        points_found: report.fixed_points.len(),
    })
}

/// A human-readable name for a linear-stability classification verdict.
pub fn classification_label(classification: Classification) -> &'static str {
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

/// Renders a complex value as `a + b i` with a signed imaginary part, using the
/// shared human-facing number formatter.
pub fn render_complex(value: &Complex) -> String {
    let sign = if value.im < 0.0 { '-' } else { '+' };
    format!("{} {sign} {}i", format_number(value.re), format_number(value.im.abs()))
}

/// Renders a coordinate vector against the state order, e.g. `x=0, y=1`.
pub fn render_coordinates(states: &[Identifier], coordinates: &[f64]) -> String {
    states
        .iter()
        .zip(coordinates)
        .map(|(state, value)| format!("{}={}", state.as_str(), format_number(*value)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A dense matrix as stable JSON: a list of rows, each a list of 17-digit floats.
pub fn matrix_json(matrix: &Matrix) -> String {
    let rows: Vec<String> = (0..matrix.rows())
        .map(|row| {
            let cells: Vec<String> =
                (0..matrix.cols()).map(|col| format!("{:.17e}", matrix.get(row, col))).collect();
            format!("[{}]", cells.join(", "))
        })
        .collect();
    format!("[{}]", rows.join(", "))
}

/// Minimal JSON string escaping (mirrors the convention in `stability.rs`).
pub fn json_string(value: &str) -> String {
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
pub fn parse_search_box(value: &str) -> Result<Vec<(f64, f64)>, String> {
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

/// Parses `MIN:MAX` into an ordered parameter range.
pub fn parse_range(value: &str) -> Result<(f64, f64), String> {
    let (min, max) = value
        .split_once(':')
        .ok_or_else(|| format!("expected MIN:MAX in --range, got '{value}'"))?;
    let min = parse_number(min.trim(), "--range minimum")?;
    let max = parse_number(max.trim(), "--range maximum")?;
    if min > max {
        return Err(format!("--range '{value}' has minimum above maximum"));
    }
    Ok((min, max))
}

/// Parses a comma-separated list of identifiers, rejecting an empty list.
pub fn parse_identifiers(value: &str) -> Result<Vec<Identifier>, String> {
    let identifiers = value
        .split(',')
        .map(|item| Identifier::new(item.trim()).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if identifiers.is_empty() {
        Err("expected at least one identifier".to_owned())
    } else {
        Ok(identifiers)
    }
}

/// Parses a comma-separated list of complex poles, each `RE` or `RE:IM`.
pub fn parse_poles(value: &str) -> Result<Vec<Complex>, String> {
    let poles = value
        .split(',')
        .map(|entry| {
            let entry = entry.trim();
            match entry.split_once(':') {
                Some((re, im)) => Ok(Complex::new(
                    parse_number(re.trim(), "--poles real part")?,
                    parse_number(im.trim(), "--poles imaginary part")?,
                )),
                None => Ok(Complex::real(parse_number(entry, "--poles value")?)),
            }
        })
        .collect::<Result<Vec<_>, String>>()?;
    if poles.is_empty() {
        Err("expected at least one pole in --poles".to_owned())
    } else {
        Ok(poles)
    }
}

/// A `NAME=VALUE` numeric assignment.
pub fn parse_assignment(value: &str) -> Result<(Identifier, f64), String> {
    let (name, number) =
        value.split_once('=').ok_or_else(|| format!("expected NAME=VALUE, got '{value}'"))?;
    let identifier = Identifier::new(name.trim()).map_err(|error| error.to_string())?;
    Ok((identifier, parse_number(number.trim(), "assignment value")?))
}

/// Parses a comma-separated `NAME=VALUE[,NAME=VALUE...]` assignment list and
/// orders the values against `states`, producing one value per state.
///
/// Every state must be assigned exactly once and no unknown name may appear, so
/// the returned vector is a faithful, state-ordered image of the assignments —
/// the shape [`lawsynth_lyapunov`] and [`lawsynth_mpc`] expect for an initial
/// condition or setpoint. `flag` names the originating option for error text.
pub fn parse_state_vector(
    value: &str,
    states: &[Identifier],
    flag: &str,
) -> Result<Vec<f64>, String> {
    let mut assignments: Vec<(Identifier, f64)> = Vec::new();
    for entry in value.split(',') {
        let (name, number) = parse_assignment(entry)?;
        if assignments.iter().any(|(existing, _)| existing == &name) {
            return Err(format!("{flag} assigns '{}' more than once", name.as_str()));
        }
        assignments.push((name, number));
    }
    let mut ordered = Vec::with_capacity(states.len());
    for state in states {
        let value = assignments
            .iter()
            .find(|(name, _)| name == state)
            .map(|(_, value)| *value)
            .ok_or_else(|| {
                format!("{flag} is missing an assignment for state '{}'", state.as_str())
            })?;
        ordered.push(value);
    }
    if let Some((name, _)) = assignments.iter().find(|(name, _)| !states.contains(name)) {
        return Err(format!(
            "{flag} names '{}', which is not a state of this world",
            name.as_str()
        ));
    }
    Ok(ordered)
}

/// Parses a finite floating-point value, tagging the flag for error messages.
pub fn parse_number(value: &str, flag: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}' for {flag}"))?;
    if number.is_finite() {
        Ok(number)
    } else {
        Err(format!("value '{value}' for {flag} must be finite"))
    }
}

/// Parses a strictly positive finite value.
pub fn parse_positive(value: &str, flag: &str) -> Result<f64, String> {
    let number = parse_number(value, flag)?;
    if number > 0.0 { Ok(number) } else { Err(format!("value for {flag} must be > 0")) }
}

/// Parses a non-negative integer count.
pub fn parse_usize(value: &str, flag: &str) -> Result<usize, String> {
    value.parse().map_err(|_| format!("invalid count '{value}' for {flag}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_range_in_order() {
        assert_eq!(parse_range("-1:2").unwrap(), (-1.0, 2.0));
        assert!(parse_range("2:1").unwrap_err().contains("minimum above maximum"));
    }

    #[test]
    fn parses_real_and_complex_poles() {
        let poles = parse_poles("-1,-2:0.5,-2:-0.5").unwrap();
        assert_eq!(poles.len(), 3);
        assert_eq!(poles[0], Complex::real(-1.0));
        assert_eq!(poles[1], Complex::new(-2.0, 0.5));
        assert_eq!(poles[2], Complex::new(-2.0, -0.5));
    }

    #[test]
    fn matrix_json_is_row_major() {
        let matrix = Matrix::from_rows(vec![vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
        let json = matrix_json(&matrix);
        assert!(json.starts_with("[["));
        assert!(json.contains("1.00000000000000000e0"));
    }
}
