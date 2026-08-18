//! Qualitative dynamics analysis section for the world report.
//!
//! This section answers "is this world stable, chaotic, or conservative?" by
//! running three deterministic, offline analysis engines over the world's
//! autonomous field `ẋ = f(x)`:
//!
//! - **fixed points & stability** ([`lawsynth_stability`]) — where the flow rests
//!   and the linear-stability verdict at each equilibrium;
//! - **chaos** ([`lawsynth_lyapunov`]) — the largest Lyapunov exponent, a
//!   time-averaged estimate whose sign reads sensitive dependence; and
//! - **conserved quantities** ([`lawsynth_invariants`]) — polynomial invariants of
//!   the flow, when any exist in the candidate library.
//!
//! Every engine is deterministic, so the rendered HTML is byte-stable for a given
//! world. The section is honest about its limits: search boxes are stated,
//! non-hyperbolic points are flagged inconclusive, the exponent is called an
//! estimate, and an empty invariant basis is reported as library-bounded absence
//! rather than proof.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, symbols};
use lawsynth_sim::Trajectory;
use lawsynth_stability::{Classification, Complex, StabilityConfig, analyze_stability};
use lawsynth_world::World;

use lawsynth_invariants::{InvariantConfig, detect_invariants};
use lawsynth_lyapunov::{LyapunovConfig, lyapunov_spectrum};

use crate::html::escape;
use crate::render::format_number;
use crate::theme::Theme;

/// Number of integration steps used for the Lyapunov estimate. Long enough for a
/// usable, deterministic estimate on the small worlds a report describes (at the
/// engine's default step this is ~180 post-transient time units) while keeping
/// rendering fast.
const LYAPUNOV_STEPS: usize = 20_000;

/// Half-width of the band around zero within which the largest Lyapunov exponent
/// is read as "neutral/conservative" rather than expanding or contracting.
///
/// A finite-time estimate cannot resolve an exponent from zero below roughly the
/// inverse of the averaging window, and conservative flows with orbit-period
/// shear (e.g. Lotka–Volterra) hold a small *positive* finite-time largest
/// exponent that only decays like `ln(T)/T`. This band is set above that floor so
/// such systems read as neutral rather than falsely chaotic, while genuine chaos
/// (largest exponent of order `0.1`–`1`, e.g. Lorenz) still clears it.
const NEUTRAL_BAND: f64 = 5e-2;

/// Default per-state search interval when the trajectory yields no usable range.
const DEFAULT_BOX: (f64, f64) = (-2.0, 2.0);

/// A world's laws with every declared parameter replaced by its constant value,
/// aligned one-to-one with `states`, yielding a parameter-free autonomous field.
///
/// Returns `None` when a state has no law (the field would be under-determined).
/// Mirrors the CLI's `autonomous_fields` conversion but is self-contained so the
/// report crate does not depend on `lawsynth-cli` or `lawsynth-bifurcation`.
fn autonomous_fields(world: &World, states: &[Identifier]) -> Option<Vec<(Identifier, Expr)>> {
    let mut fields = Vec::with_capacity(states.len());
    for state in states {
        let law = world.laws().get(state)?;
        let mut expression = law.expression.clone();
        for (name, parameter) in world.parameters() {
            expression = substitute(&expression, name, parameter.value);
        }
        fields.push((state.clone(), expression));
    }
    Some(fields)
}

/// Returns a copy of `expression` with every `parameter` symbol replaced by the
/// constant `value` — a pure structural rewrite that never mutates the input.
fn substitute(expression: &Expr, parameter: &Identifier, value: f64) -> Expr {
    match expression {
        Expr::Constant(constant) => Expr::constant(*constant),
        Expr::Symbol(identifier) => {
            if identifier == parameter {
                Expr::constant(value)
            } else {
                Expr::symbol(identifier.clone())
            }
        }
        Expr::Unary { operator, operand } => {
            Expr::unary(*operator, substitute(operand, parameter, value))
        }
        Expr::Binary { operator, left, right } => Expr::binary(
            *operator,
            substitute(left, parameter, value),
            substitute(right, parameter, value),
        ),
    }
}

/// Any symbols the substituted fields reference that are not states — the free
/// symbols that make the field non-autonomous and unanalyzable here.
fn free_symbols(fields: &[(Identifier, Expr)], states: &[Identifier]) -> BTreeSet<Identifier> {
    let state_set: BTreeSet<&Identifier> = states.iter().collect();
    let mut free = BTreeSet::new();
    for (_, expression) in fields {
        for symbol in symbols(expression) {
            if !state_set.contains(&symbol) {
                free.insert(symbol);
            }
        }
    }
    free
}

/// A human-readable name for a linear-stability classification verdict, carrying
/// the honest "inconclusive" note where the linearization cannot decide.
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

/// Renders a complex eigenvalue as `a + b i` with a signed imaginary part.
fn render_complex(value: &Complex) -> String {
    let sign = if value.im < 0.0 { '-' } else { '+' };
    format!("{} {sign} {}i", format_number(value.re), format_number(value.im.abs()))
}

/// Renders a coordinate vector against the state order, e.g. `x=0, y=1`.
fn render_coordinates(states: &[Identifier], coordinates: &[f64]) -> String {
    states
        .iter()
        .zip(coordinates)
        .map(|(state, value)| format!("{}={}", state.as_str(), format_number(*value)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Derives a data-appropriate search box from the simulated trajectory: the
/// per-state min/max widened by a margin so a nearby equilibrium is inside it.
///
/// Falls back to [`DEFAULT_BOX`] for any state with no usable trajectory range.
/// Returned intervals are in `states` order.
fn search_box(trajectory: &Trajectory, states: &[Identifier]) -> Vec<(f64, f64)> {
    states
        .iter()
        .map(|state| {
            let Some(values) = trajectory.values.get(state) else {
                return DEFAULT_BOX;
            };
            let mut lo = f64::INFINITY;
            let mut hi = f64::NEG_INFINITY;
            for &value in values {
                if value.is_finite() {
                    lo = lo.min(value);
                    hi = hi.max(value);
                }
            }
            if !lo.is_finite() || !hi.is_finite() {
                return DEFAULT_BOX;
            }
            let margin = (0.2 * (hi - lo)).max(0.5);
            (lo - margin, hi + margin)
        })
        .collect()
}

/// Renders one search box as `x ∈ [lo, hi], y ∈ [lo, hi]` for state ordering.
fn render_box(states: &[Identifier], search: &[(f64, f64)]) -> String {
    states
        .iter()
        .zip(search)
        .map(|(state, (lo, hi))| {
            format!(
                "{} &isin; [{}, {}]",
                escape(state.as_str()),
                format_number(*lo),
                format_number(*hi)
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The Lyapunov initial condition: the trajectory's first sample per state,
/// falling back to `default_initial` for any state with no trajectory column.
fn initial_condition(
    trajectory: &Trajectory,
    states: &[Identifier],
    default_initial: f64,
) -> Vec<f64> {
    states
        .iter()
        .map(|state| {
            trajectory
                .values
                .get(state)
                .and_then(|values| values.first().copied())
                .unwrap_or(default_initial)
        })
        .collect()
}

/// Renders a detected invariant as a readable combination of basis terms, e.g.
/// `x^2 + y^2`, scaling coefficients so the largest-magnitude term reads as 1.
fn render_invariant_terms(labels: &[String], coefficients: &[f64]) -> String {
    let max_abs = coefficients.iter().fold(0.0_f64, |acc, coefficient| acc.max(coefficient.abs()));
    if max_abs == 0.0 {
        return "0".to_owned();
    }
    let mut out = String::new();
    let mut first = true;
    for (label, &coefficient) in labels.iter().zip(coefficients) {
        // Drop terms that are negligible relative to the dominant one.
        if coefficient.abs() < 1e-6 * max_abs {
            continue;
        }
        let scaled = coefficient / max_abs;
        let magnitude = scaled.abs();
        if first {
            if scaled < 0.0 {
                out.push('-');
            }
            first = false;
        } else if scaled < 0.0 {
            out.push_str(" - ");
        } else {
            out.push_str(" + ");
        }
        if (magnitude - 1.0).abs() < 1e-9 {
            out.push_str(&escape(label));
        } else {
            let _ = write!(out, "{} {}", format_number(magnitude), escape(label));
        }
    }
    if out.is_empty() { "0".to_owned() } else { out }
}

/// Renders the "Dynamics analysis" section into `body`.
///
/// Skips (with an honest one-line note) when the world has no states, a state
/// lacks a law, or the field references undeclared symbols after parameter
/// substitution (non-autonomous). Otherwise runs the three engines and renders
/// their honest verdicts. Reuses the already-computed `trajectory` for the
/// stability search box and the Lyapunov initial condition, so nothing new is
/// simulated and the output stays byte-stable.
pub(crate) fn dynamics_analysis_section(
    body: &mut String,
    world: &World,
    trajectory: &Trajectory,
    default_initial: f64,
    _theme: &Theme,
) {
    body.push_str("  <section>\n    <h2>Dynamics analysis</h2>\n");

    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    if states.is_empty() {
        body.push_str(
            "    <p class=\"muted\">Skipped: this world declares no state variables, so it has no autonomous flow to analyze.</p>\n  </section>\n",
        );
        return;
    }

    let Some(fields) = autonomous_fields(world, &states) else {
        body.push_str(
            "    <p class=\"muted\">Skipped: at least one state has no law, so the field is under-determined.</p>\n  </section>\n",
        );
        return;
    };

    let free = free_symbols(&fields, &states);
    if !free.is_empty() {
        let names: Vec<String> = free.iter().map(|symbol| escape(symbol.as_str())).collect();
        let _ = writeln!(
            body,
            "    <p class=\"muted\">Skipped: after substituting declared parameters the field still references undeclared symbol(s) [{}], so it is non-autonomous and cannot be analyzed as &#7819; = f(x).</p>\n  </section>",
            names.join(", ")
        );
        return;
    }

    body.push_str(
        "    <p class=\"muted\">Qualitative verdicts over the autonomous field &#7819; = f(x) (declared parameters pinned to their values). All three engines are deterministic and offline.</p>\n",
    );

    fixed_points_part(body, &fields, &states, trajectory);
    lyapunov_part(body, &fields, &states, trajectory, default_initial);
    invariants_part(body, &fields, &states);

    body.push_str("  </section>\n");
}

/// Fixed points & stability sub-part.
fn fixed_points_part(
    body: &mut String,
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    trajectory: &Trajectory,
) {
    body.push_str("    <h3>Fixed points &amp; stability</h3>\n");
    let search = search_box(trajectory, states);
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Newton search over a box derived from the trajectory range: {}.</p>",
        render_box(states, &search)
    );

    let config = StabilityConfig::new(search);
    let report = match analyze_stability(fields, states, &config) {
        Ok(report) => report,
        Err(error) => {
            let _ = writeln!(
                body,
                "    <p class=\"muted\">Fixed-point search unavailable: {}.</p>",
                escape(&error.to_string())
            );
            return;
        }
    };

    if report.fixed_points.is_empty() {
        let _ = writeln!(
            body,
            "    <p class=\"muted\">No fixed point located inside the box ({} of {} deterministic seeds converged). Absence in this box is not proof that the system has none.</p>",
            report.seeds_converged, report.seeds_total
        );
        return;
    }

    body.push_str("    <table>\n      <thead><tr><th>Coordinates</th><th>Classification</th><th>Eigenvalues</th></tr></thead>\n      <tbody>\n");
    for point in &report.fixed_points {
        let eigenvalues =
            point.eigenvalues.iter().map(render_complex).collect::<Vec<_>>().join(", ");
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td>{}</td><td class=\"mono\">{}</td></tr>",
            escape(&render_coordinates(states, &point.coordinates)),
            classification_label(point.classification),
            escape(&eigenvalues)
        );
    }
    body.push_str("      </tbody>\n    </table>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">{} distinct fixed point(s); {} of {} seeds converged. Center/marginal verdicts are inconclusive — the linearization cannot decide there.</p>",
        report.fixed_points.len(),
        report.seeds_converged,
        report.seeds_total
    );
}

/// Chaos (largest Lyapunov exponent) sub-part.
fn lyapunov_part(
    body: &mut String,
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    trajectory: &Trajectory,
    default_initial: f64,
) {
    body.push_str("    <h3>Chaos &mdash; largest Lyapunov exponent</h3>\n");
    let initial = initial_condition(trajectory, states, default_initial);
    let config = LyapunovConfig::default().with_steps(LYAPUNOV_STEPS);
    let report = match lyapunov_spectrum(fields, states, &initial, &config) {
        Ok(report) => report,
        Err(error) => {
            let _ = writeln!(
                body,
                "    <p class=\"muted\">Lyapunov estimate unavailable: {}.</p>",
                escape(&error.to_string())
            );
            return;
        }
    };

    let largest = report.largest();
    let verdict = if largest > NEUTRAL_BAND {
        "largest exponent &gt; 0 &rarr; sensitive dependence on initial conditions (chaotic)"
    } else if largest < -NEUTRAL_BAND {
        "largest exponent &lt; 0 &rarr; trajectories contract (dissipative / non-chaotic)"
    } else {
        "largest exponent &asymp; 0 &rarr; neutral (conservative / marginal &mdash; no sensitive dependence resolved)"
    };

    let _ = writeln!(
        body,
        "    <p class=\"muted\">From initial condition {}, integrated {} steps.</p>",
        escape(&render_coordinates(states, &initial)),
        LYAPUNOV_STEPS
    );
    body.push_str("    <table>\n      <thead><tr><th>Quantity</th><th>Value</th></tr></thead>\n      <tbody>\n");
    let _ = writeln!(
        body,
        "        <tr><td>Largest exponent &lambda;&#8321;</td><td class=\"mono\">{}</td></tr>",
        format_number(largest)
    );
    let _ = writeln!(
        body,
        "        <tr><td>Sum &Sigma;&lambda; (mean divergence)</td><td class=\"mono\">{}</td></tr>",
        format_number(report.sum())
    );
    let _ = writeln!(
        body,
        "        <tr><td>Kaplan&ndash;Yorke dimension</td><td class=\"mono\">{}</td></tr>",
        format_number(report.kaplan_yorke_dimension())
    );
    body.push_str("      </tbody>\n    </table>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Verdict: {verdict}. This is a time-averaged estimate (accuracy grows with integration length); |&lambda;&#8321;| below {} is read as unresolved from zero. The sum &Sigma;&lambda; is the tightest quantity: negative means a volume-contracting (dissipative) flow, &asymp; 0 a conservative one.</p>",
        format_number(NEUTRAL_BAND)
    );
}

/// Conserved quantities sub-part.
fn invariants_part(body: &mut String, fields: &[(Identifier, Expr)], states: &[Identifier]) {
    body.push_str("    <h3>Conserved quantities</h3>\n");
    let config = InvariantConfig::default();
    let report = match detect_invariants(fields, states, &config) {
        Ok(report) => report,
        Err(error) => {
            let _ = writeln!(
                body,
                "    <p class=\"muted\">Invariant search unavailable: {}.</p>",
                escape(&error.to_string())
            );
            return;
        }
    };

    if report.invariants.is_empty() {
        body.push_str(
            "    <p class=\"muted\">No conserved quantity found in the polynomial basis (degree &le; 2). The library is bounded, so absence here is not proof that none exists.</p>\n",
        );
        return;
    }

    body.push_str("    <table>\n      <thead><tr><th>Conserved quantity H(x)</th><th>Residual &#8214;L&#7584;H&#8214;</th></tr></thead>\n      <tbody>\n");
    for invariant in &report.invariants {
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td></tr>",
            render_invariant_terms(&report.basis_labels, &invariant.coefficients),
            format_number(invariant.residual)
        );
    }
    body.push_str("      </tbody>\n    </table>\n");
    body.push_str(
        "    <p class=\"muted\">Each row is a hypothesis: a combination whose Lie derivative nearly vanishes on the sample grid, not a proof of exact conservation.</p>\n",
    );
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::{Expr, UnaryOperator};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn renders_terms_scaled_to_leading_one() {
        let labels = vec!["x^2".to_owned(), "x*y".to_owned(), "y^2".to_owned()];
        // Equal weight on the two squares, scaled to a leading 1 → "x^2 + y^2".
        let weight = std::f64::consts::FRAC_1_SQRT_2;
        let coefficients = vec![weight, 0.0, weight];
        assert_eq!(render_invariant_terms(&labels, &coefficients), "x^2 + y^2");
    }

    #[test]
    fn free_symbols_flags_undeclared() {
        let x = id("x");
        let a = id("a");
        let fields =
            vec![(x.clone(), Expr::product(Expr::symbol(a.clone()), Expr::symbol(x.clone())))];
        let free = free_symbols(&fields, &[x]);
        assert!(free.contains(&a));
    }

    #[test]
    fn substitute_replaces_only_the_parameter() {
        let x = id("x");
        let k = id("k");
        let expression = Expr::product(Expr::symbol(k.clone()), Expr::symbol(x.clone()));
        let bound = substitute(&expression, &k, 2.0);
        assert_eq!(bound, Expr::product(Expr::constant(2.0), Expr::symbol(x)));
    }

    #[test]
    fn search_box_widens_the_trajectory_range() {
        let x = id("x");
        let trajectory = Trajectory {
            time: vec![0.0, 1.0],
            values: [(x.clone(), vec![-1.0, 1.0])].into_iter().collect(),
        };
        let search = search_box(&trajectory, &[x]);
        assert_eq!(search.len(), 1);
        let (lo, hi) = search[0];
        assert!(lo < -1.0 && hi > 1.0, "box {lo}..{hi} must enclose the range");
    }

    #[test]
    fn classification_labels_flag_inconclusive() {
        assert!(classification_label(Classification::Center).contains("inconclusive"));
        assert_eq!(classification_label(Classification::StableSpiral), "stable spiral");
    }

    #[test]
    fn negate_field_is_autonomous() {
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let fields = vec![(x.clone(), field)];
        assert!(free_symbols(&fields, &[x]).is_empty());
    }
}
