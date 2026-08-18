//! Side-by-side HTML diff of two worlds, reusing the report crate's styling.
//!
//! [`render_comparison`] produces a single, dependency-free HTML file that puts
//! two worlds next to each other — laws, parameters, and complexity — so model
//! selection is legible at a glance. Shared expression rendering keeps the
//! equations identical to the trajectory report and the `export` artifacts.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sim::Trajectory;
use lawsynth_world::World;

use crate::analysis::{ChaosVerdict, analyze_world};
use crate::html::{document, escape};
use crate::render::{format_number, render_continuous_law};
use crate::theme::Theme;
use crate::{ReportOptions, simulate_default};

/// Rendering and simulation options for a world-comparison report.
///
/// Mirrors the single-world [`ReportOptions`] for the fields the comparison
/// needs: the [`Theme`], the `include_dynamics` gate for the Dynamics diff, and
/// the simulation window used to derive each world's stability search box and
/// Lyapunov initial condition. Defaults match [`ReportOptions::default`] so the
/// two reports describe the same nominal trajectory.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonOptions {
    /// Brand theme applied to the document and its tables.
    pub theme: Theme,
    /// Whether to render the "Dynamics analysis" comparison section.
    ///
    /// Defaults to `true`. Set to `false` to omit it entirely — the output is
    /// then byte-identical to the pre-feature comparison (summary + laws +
    /// parameters diffs only).
    pub include_dynamics: bool,
    /// Inclusive simulation start time (for each world's search box).
    pub start: f64,
    /// Inclusive simulation end time.
    pub end: f64,
    /// Maximum integration step.
    pub step: f64,
    /// Initial value for any state without an explicit trajectory sample.
    pub default_initial: f64,
}

impl Default for ComparisonOptions {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            include_dynamics: true,
            start: 0.0,
            end: 10.0,
            step: 0.1,
            default_initial: 1.0,
        }
    }
}

/// Renders a self-contained side-by-side comparison of two worlds.
///
/// `label_a` and `label_b` head the two columns (typically the source paths).
/// Uses [`ComparisonOptions::default`] (brand light theme, Dynamics diff on);
/// see [`render_comparison_with_options`] to override either.
pub fn render_comparison(world_a: &World, label_a: &str, world_b: &World, label_b: &str) -> String {
    render_comparison_with_options(
        world_a,
        label_a,
        world_b,
        label_b,
        &ComparisonOptions::default(),
    )
}

/// [`render_comparison`] with an explicit [`Theme`].
pub fn render_comparison_with_theme(
    world_a: &World,
    label_a: &str,
    world_b: &World,
    label_b: &str,
    theme: Theme,
) -> String {
    render_comparison_with_options(
        world_a,
        label_a,
        world_b,
        label_b,
        &ComparisonOptions { theme, ..ComparisonOptions::default() },
    )
}

/// [`render_comparison`] with explicit [`ComparisonOptions`].
///
/// The Dynamics analysis diff is appended after the laws and parameters diffs
/// only when [`ComparisonOptions::include_dynamics`] is set.
pub fn render_comparison_with_options(
    world_a: &World,
    label_a: &str,
    world_b: &World,
    label_b: &str,
    options: &ComparisonOptions,
) -> String {
    let title = format!("LawSynth comparison: {label_a} vs {label_b}");
    let mut body = String::new();

    let complexity_a = total_complexity(world_a);
    let complexity_b = total_complexity(world_b);

    let _ = write!(
        body,
        "  <header>\n    <h1>World comparison</h1>\n    <p class=\"subtitle\">A &middot; {} &nbsp;|&nbsp; B &middot; {}</p>\n  </header>\n",
        escape(label_a),
        escape(label_b)
    );

    // Summary row.
    body.push_str("  <section>\n    <h2>Summary</h2>\n");
    body.push_str("    <table>\n      <thead><tr><th>Metric</th><th>A</th><th>B</th><th>&Delta;</th></tr></thead>\n      <tbody>\n");
    summary_row(
        &mut body,
        "State variables",
        world_a.state_ids().count(),
        world_b.state_ids().count(),
    );
    summary_row(&mut body, "Parameters", world_a.parameters().len(), world_b.parameters().len());
    summary_row(&mut body, "Laws", world_a.laws().len(), world_b.laws().len());
    summary_row(&mut body, "Complexity (nodes)", complexity_a, complexity_b);
    body.push_str("      </tbody>\n    </table>\n  </section>\n");

    laws_diff_section(&mut body, world_a, world_b);
    parameters_diff_section(&mut body, world_a, world_b);
    if options.include_dynamics {
        dynamics_comparison_section(&mut body, world_a, world_b, options);
    }

    document(&title, &body, &options.theme)
}

/// Simulates a world's default trajectory, degrading to an empty trajectory when
/// the world cannot be forward-simulated (e.g. a non-autonomous field). An empty
/// trajectory makes [`analyze_world`] fall back to its default search box and
/// initial condition, which the non-autonomous path never reaches anyway.
fn simulate_or_empty(world: &World, options: &ComparisonOptions) -> Trajectory {
    let sim_options = ReportOptions {
        start: options.start,
        end: options.end,
        step: options.step,
        default_initial: options.default_initial,
        ..ReportOptions::default()
    };
    simulate_default(world, &sim_options)
        .unwrap_or_else(|_| Trajectory { time: Vec::new(), values: BTreeMap::new() })
}

/// Renders the side-by-side "Dynamics analysis" comparison into `body`.
///
/// Reuses the shared [`analyze_world`] helper on each world so the verdicts match
/// the single-world report exactly. Each aspect is one row with A and B cells and
/// a diff status; the stability-class and chaos-verdict rows are marked
/// `.changed` when the two worlds disagree. A world that is not an autonomous ODE
/// shows an honest one-line note in its cells rather than failing the report.
fn dynamics_comparison_section(
    body: &mut String,
    world_a: &World,
    world_b: &World,
    options: &ComparisonOptions,
) {
    let trajectory_a = simulate_or_empty(world_a, options);
    let trajectory_b = simulate_or_empty(world_b, options);
    let a = analyze_world(world_a, &trajectory_a, options.default_initial).summary();
    let b = analyze_world(world_b, &trajectory_b, options.default_initial).summary();

    body.push_str("  <section>\n    <h2>Dynamics analysis</h2>\n");
    body.push_str(
        "    <p class=\"muted\">Side-by-side qualitative verdicts over each world's autonomous field &#7819; = f(x) (declared parameters pinned to their values). Deterministic and offline; a world that is not an autonomous ODE is marked as such.</p>\n",
    );
    body.push_str("    <table>\n      <thead><tr><th>Aspect</th><th>A</th><th>B</th><th>Status</th></tr></thead>\n      <tbody>\n");

    // Plain-text cells are escaped; entity-bearing cells (search box, invariants,
    // and the skip note) are passed through verbatim.
    let note = "not analyzable as &#7819; = f(x)";
    dynamics_row(
        body,
        "Fixed points",
        &escaped_cell(&a.fixed_points, note),
        &escaped_cell(&b.fixed_points, note),
        NEUTRAL_STATUS,
    );
    dynamics_row(
        body,
        "Stability class",
        &escaped_cell(&a.stability_class, note),
        &escaped_cell(&b.stability_class, note),
        diff_status(&a.stability_key, &b.stability_key),
    );
    dynamics_row(
        body,
        "Search box",
        &raw_cell(&a.search_box, note),
        &raw_cell(&b.search_box, note),
        NEUTRAL_STATUS,
    );
    dynamics_row(
        body,
        "Largest Lyapunov &lambda;&#8321;",
        &escaped_cell(&a.lyapunov_value, note),
        &escaped_cell(&b.lyapunov_value, note),
        NEUTRAL_STATUS,
    );
    dynamics_row(
        body,
        "Chaos verdict",
        &escaped_cell(&a.chaos_verdict, note),
        &escaped_cell(&b.chaos_verdict, note),
        chaos_status(a.chaos_key, b.chaos_key),
    );
    dynamics_row(
        body,
        "Conserved quantities",
        &raw_cell(&a.invariants, note),
        &raw_cell(&b.invariants, note),
        NEUTRAL_STATUS,
    );

    body.push_str("      </tbody>\n    </table>\n");
    body.push_str(
        "    <p class=\"muted\">Center/marginal classifications are inconclusive; the largest Lyapunov exponent is a finite-time estimate (a magnitude below the neutral band reads as zero); conserved quantities are bounded by the degree &le; 2 candidate library, so absence is not proof.</p>\n  </section>\n",
    );
}

/// The status for an always-informational (non-diffed) row.
const NEUTRAL_STATUS: (&str, &str) = ("neutral", "&mdash;");

/// A cell for a plain-text summary field, HTML-escaped, or the raw skip note.
fn escaped_cell(value: &Option<String>, note: &str) -> String {
    match value {
        Some(text) => escape(text),
        None => note.to_owned(),
    }
}

/// A cell for a summary field that already carries HTML entities, passed through
/// verbatim, or the raw skip note.
fn raw_cell(value: &Option<String>, note: &str) -> String {
    match value {
        Some(text) => text.clone(),
        None => note.to_owned(),
    }
}

/// Diff status for two comparison keys: `.changed` when both are present and
/// differ, `same` when both are present and equal, `&mdash;` when either world
/// is not analyzable (so the aspect cannot be compared).
fn diff_status(a: &Option<String>, b: &Option<String>) -> (&'static str, &'static str) {
    match (a, b) {
        (Some(x), Some(y)) if x != y => ("changed", "changed"),
        (Some(_), Some(_)) => ("neutral", "same"),
        _ => NEUTRAL_STATUS,
    }
}

/// Diff status for the two chaos verdicts, marked `.changed` when they differ.
fn chaos_status(a: Option<ChaosVerdict>, b: Option<ChaosVerdict>) -> (&'static str, &'static str) {
    match (a, b) {
        (Some(x), Some(y)) if x != y => ("changed", "changed"),
        (Some(_), Some(_)) => ("neutral", "same"),
        _ => NEUTRAL_STATUS,
    }
}

/// Writes one aspect row: label, A cell, B cell, and a themed status column.
fn dynamics_row(body: &mut String, aspect: &str, a: &str, b: &str, status: (&str, &str)) {
    let _ = writeln!(
        body,
        "        <tr><td>{aspect}</td><td class=\"mono\">{a}</td><td class=\"mono\">{b}</td><td class=\"{}\">{}</td></tr>",
        status.0, status.1
    );
}

fn summary_row(body: &mut String, label: &str, a: usize, b: usize) {
    let delta = b as isize - a as isize;
    let class = if delta == 0 { "neutral" } else { "changed" };
    let _ = writeln!(
        body,
        "        <tr><td>{}</td><td class=\"mono\">{a}</td><td class=\"mono\">{b}</td><td class=\"mono {class}\">{delta:+}</td></tr>",
        escape(label)
    );
}

fn laws_diff_section(body: &mut String, world_a: &World, world_b: &World) {
    body.push_str("  <section>\n    <h2>Laws</h2>\n");
    body.push_str("    <table>\n      <thead><tr><th>State</th><th>A</th><th>B</th><th>Status</th></tr></thead>\n      <tbody>\n");
    for id in union_ids(world_a.laws().keys(), world_b.laws().keys()) {
        let left = world_a.laws().get(&id);
        let right = world_b.laws().get(&id);
        let render = |law: Option<&lawsynth_world::ContinuousLaw>| match law {
            Some(law) => escape(&render_continuous_law(id.as_str(), &law.expression)),
            None => "&mdash;".to_owned(),
        };
        let status = match (left, right) {
            (None, Some(_)) => ("added", "added"),
            (Some(_), None) => ("removed", "removed"),
            (Some(l), Some(r))
                if l.expression.to_canonical_string() != r.expression.to_canonical_string() =>
            {
                ("changed", "changed")
            }
            _ => ("neutral", "identical"),
        };
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td class=\"equation-cell\">{}</td><td class=\"equation-cell\">{}</td><td class=\"{}\">{}</td></tr>",
            escape(id.as_str()),
            render(left),
            render(right),
            status.0,
            status.1
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
}

fn parameters_diff_section(body: &mut String, world_a: &World, world_b: &World) {
    body.push_str("  <section>\n    <h2>Parameters</h2>\n");
    if world_a.parameters().is_empty() && world_b.parameters().is_empty() {
        body.push_str(
            "    <p class=\"muted\">Neither world carries free parameters.</p>\n  </section>\n",
        );
        return;
    }
    body.push_str("    <table>\n      <thead><tr><th>Name</th><th>A</th><th>B</th><th>Status</th></tr></thead>\n      <tbody>\n");
    for id in union_ids(world_a.parameters().keys(), world_b.parameters().keys()) {
        let left = world_a.parameters().get(&id);
        let right = world_b.parameters().get(&id);
        let render = |parameter: Option<&lawsynth_world::Parameter>| match parameter {
            Some(parameter) => format_number(parameter.value),
            None => "&mdash;".to_owned(),
        };
        let status = match (left, right) {
            (None, Some(_)) => ("added", "added"),
            (Some(_), None) => ("removed", "removed"),
            (Some(l), Some(r)) if l.value != r.value || l.unit != r.unit => ("changed", "changed"),
            _ => ("neutral", "identical"),
        };
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"{}\">{}</td></tr>",
            escape(id.as_str()),
            render(left),
            render(right),
            status.0,
            status.1
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
    // Diff-status classes (.added/.removed/.changed/.neutral/.equation-cell) are
    // defined once in the themed stylesheet, so no inline <style> is needed here.
}

fn union_ids<'a>(
    left: impl Iterator<Item = &'a Identifier>,
    right: impl Iterator<Item = &'a Identifier>,
) -> BTreeSet<Identifier> {
    left.chain(right).cloned().collect()
}

fn complexity(expression: &Expr) -> usize {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => 1,
        Expr::Unary { operand, .. } => 1 + complexity(operand),
        Expr::Binary { left, right, .. } => 1 + complexity(left) + complexity(right),
    }
}

fn total_complexity(world: &World) -> usize {
    world.laws().values().map(|law| complexity(&law.expression)).sum()
}

#[cfg(test)]
mod tests {
    use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn world(coefficient: f64) -> World {
        World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), coefficient)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("k")), Expr::symbol(id("x"))),
            )],
        )
        .unwrap()
    }

    #[test]
    fn comparison_is_a_self_contained_document() {
        let html = render_comparison(&world(1.0), "a.lsworld", &world(2.0), "b.lsworld");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("World comparison"));
        assert!(html.contains("Complexity"));
        assert!(html.contains("changed"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn identical_worlds_report_no_changes() {
        let html = render_comparison(&world(1.0), "a", &world(1.0), "b");
        assert!(html.contains("identical"));
    }

    /// Damped linear oscillator: ẋ = y, ẏ = -x - c·y with c = 0.3. A stable
    /// spiral at the origin; dissipative (Σλ = -c) with no quadratic invariant.
    fn damped_oscillator() -> World {
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            [Parameter::new(id("c"), 0.3)],
            [
                ContinuousLaw::new(id("x"), Expr::symbol(id("y"))),
                ContinuousLaw::new(
                    id("y"),
                    Expr::difference(
                        Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
                        Expr::product(Expr::symbol(id("c")), Expr::symbol(id("y"))),
                    ),
                ),
            ],
        )
        .unwrap()
    }

    /// Undamped harmonic oscillator: ẋ = y, ẏ = -x. A center at the origin
    /// (inconclusive) with conserved energy x² + y² and ≈ zero exponents.
    fn harmonic_oscillator() -> World {
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            Vec::<Parameter>::new(),
            [
                ContinuousLaw::new(id("x"), Expr::symbol(id("y"))),
                ContinuousLaw::new(
                    id("y"),
                    Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
                ),
            ],
        )
        .unwrap()
    }

    #[test]
    fn dynamics_diff_shows_both_verdicts_and_marks_differing_chaos() {
        // A = dissipative stable spiral, B = neutral conservative center.
        let html = render_comparison(&damped_oscillator(), "A", &harmonic_oscillator(), "B");
        assert!(html.contains("Dynamics analysis"), "dynamics section present");

        // Both worlds' stability classifications appear side by side.
        assert!(html.contains("stable spiral"), "A's stable spiral verdict shown");
        assert!(html.contains("center"), "B's center classification shown");

        // Both chaos verdicts appear.
        assert!(html.contains("dissipative"), "A's dissipative chaos verdict shown");
        assert!(html.contains("neutral / conservative"), "B's neutral chaos verdict shown");

        // The chaos-verdict row is marked .changed because the verdicts differ.
        assert!(
            html.contains(
                "Chaos verdict</td><td class=\"mono\">dissipative</td><td class=\"mono\">neutral / conservative</td><td class=\"changed\">changed</td>"
            ),
            "differing chaos verdicts must mark the row changed"
        );

        // B carries a conserved energy; A carries none.
        assert!(html.contains("x^2 + y^2"), "B's conserved energy shown");
        assert!(html.contains("none in the degree &le; 2 basis"), "A's honest no-invariant note");
    }

    #[test]
    fn dynamics_diff_is_deterministic() {
        let a = damped_oscillator();
        let b = harmonic_oscillator();
        assert_eq!(render_comparison(&a, "A", &b, "B"), render_comparison(&a, "A", &b, "B"));
    }

    #[test]
    fn no_dynamics_option_omits_section_and_pins_pre_feature_output() {
        let a = damped_oscillator();
        let b = harmonic_oscillator();
        let on = ComparisonOptions::default();
        let off = ComparisonOptions { include_dynamics: false, ..ComparisonOptions::default() };

        let html_on = render_comparison_with_options(&a, "A", &b, "B", &on);
        let html_off = render_comparison_with_options(&a, "A", &b, "B", &off);

        assert!(html_on.contains("Dynamics analysis"));
        assert!(!html_off.contains("Dynamics analysis"));

        // Byte-stable: the no-dynamics output is the with-dynamics output minus
        // exactly the appended section (the pre-feature comparison document).
        let marker = "  <section>\n    <h2>Dynamics analysis</h2>";
        let start = html_on.find(marker).expect("dynamics section present");
        let main_close = html_on[start..].find("</main>").expect("closing main tag") + start;
        let spliced = format!("{}{}", &html_on[..start], &html_on[main_close..]);
        assert_eq!(
            spliced, html_off,
            "no-dynamics output must be byte-identical minus the section"
        );
    }

    #[test]
    fn non_autonomous_world_shows_honest_note_without_failing() {
        // A = dx/dt = u * x references exogenous `u`: non-autonomous, so it
        // cannot be simulated or analyzed. Its cells show an honest note while
        // B (harmonic) is still fully analyzed.
        let non_autonomous = World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("u"), VariableRole::Exogenous),
            ],
            Vec::<Parameter>::new(),
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("u")), Expr::symbol(id("x"))),
            )],
        )
        .unwrap();
        let html = render_comparison(&non_autonomous, "A", &harmonic_oscillator(), "B");
        assert!(html.contains("Dynamics analysis"));
        // A's cells carry the honest one-liner.
        assert!(
            html.contains("not analyzable as &#7819; = f(x)"),
            "non-autonomous world shows the honest note"
        );
        // B is still analyzed: its neutral verdict and conserved energy appear.
        assert!(html.contains("neutral / conservative"), "B still analyzed");
        assert!(html.contains("x^2 + y^2"), "B's conserved energy still shown");
    }
}
