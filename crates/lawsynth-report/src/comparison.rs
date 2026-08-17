//! Side-by-side HTML diff of two worlds, reusing the report crate's styling.
//!
//! [`render_comparison`] produces a single, dependency-free HTML file that puts
//! two worlds next to each other — laws, parameters, and complexity — so model
//! selection is legible at a glance. Shared expression rendering keeps the
//! equations identical to the trajectory report and the `export` artifacts.

use std::collections::BTreeSet;
use std::fmt::Write;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::World;

use crate::html::{document, escape};
use crate::render::{format_number, render_continuous_law};

/// Renders a self-contained side-by-side comparison of two worlds.
///
/// `label_a` and `label_b` head the two columns (typically the source paths).
pub fn render_comparison(world_a: &World, label_a: &str, world_b: &World, label_b: &str) -> String {
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

    document(&title, &body)
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
    body.push_str(
        "  <style>.added{color:#059669;font-weight:600}.removed{color:#dc2626;font-weight:600}.changed{color:#d97706;font-weight:600}.neutral{color:#64748b}.equation-cell{font-family:'SFMono-Regular',Consolas,monospace;font-size:0.9rem}</style>\n",
    );
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
}
