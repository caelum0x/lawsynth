//! HTML page assembly for the self-contained world report.

use std::fmt::Write;

use lawsynth_expr::Expr;
use lawsynth_sim::Trajectory;
use lawsynth_world::{VariableRole, World};

use crate::render::{format_number, render_continuous_law};
use crate::svg::{line_chart, phase_portrait, series_color};

/// Escapes text for safe inclusion in HTML element content or attributes.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Counts scalar AST nodes as a deterministic complexity cost.
fn complexity(expression: &Expr) -> usize {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => 1,
        Expr::Unary { operand, .. } => 1 + complexity(operand),
        Expr::Binary { left, right, .. } => 1 + complexity(left) + complexity(right),
    }
}

fn role_name(role: VariableRole) -> &'static str {
    match role {
        VariableRole::State => "state",
        VariableRole::Control => "control",
        VariableRole::Exogenous => "exogenous",
        VariableRole::Observed => "observed",
        VariableRole::Latent => "latent",
        VariableRole::Derived => "derived",
    }
}

/// Assembles the complete standalone HTML document.
pub fn page(title: &str, world: &World, trajectory: &Trajectory) -> String {
    let mut body = String::new();
    let total_complexity: usize =
        world.laws().values().map(|law| complexity(&law.expression)).sum();

    let _ = write!(
        body,
        "  <header>\n    <h1>{}</h1>\n    <p class=\"subtitle\">Executable world &middot; {} state variable(s) &middot; complexity {}</p>\n  </header>\n",
        escape(title),
        world.state_ids().count(),
        total_complexity
    );

    laws_section(&mut body, world);
    variables_section(&mut body, world);
    parameters_section(&mut body, world);
    trajectory_section(&mut body, world, trajectory);
    phase_section(&mut body, world, trajectory);

    document(title, &body)
}

fn laws_section(body: &mut String, world: &World) {
    body.push_str("  <section>\n    <h2>Laws</h2>\n");
    body.push_str("    <div class=\"equations\">\n");
    for (target, law) in world.laws() {
        let _ = writeln!(
            body,
            "      <div class=\"equation\">{}</div>",
            escape(&render_continuous_law(target.as_str(), &law.expression))
        );
    }
    body.push_str("    </div>\n  </section>\n");
}

fn variables_section(body: &mut String, world: &World) {
    body.push_str("  <section>\n    <h2>Variables</h2>\n");
    body.push_str("    <table>\n      <thead><tr><th>Name</th><th>Role</th><th>Unit</th></tr></thead>\n      <tbody>\n");
    for variable in world.variables().values() {
        let unit = variable
            .unit
            .as_ref()
            .map(|unit| escape(unit.canonical()))
            .unwrap_or_else(|| "&mdash;".to_owned());
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td>{}</td><td>{unit}</td></tr>",
            escape(variable.id.as_str()),
            role_name(variable.role)
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
}

fn parameters_section(body: &mut String, world: &World) {
    body.push_str("  <section>\n    <h2>Parameters</h2>\n");
    if world.parameters().is_empty() {
        body.push_str("    <p class=\"muted\">No free parameters.</p>\n  </section>\n");
        return;
    }
    body.push_str("    <table>\n      <thead><tr><th>Name</th><th>Value</th><th>Unit</th></tr></thead>\n      <tbody>\n");
    for parameter in world.parameters().values() {
        let unit = parameter
            .unit
            .as_ref()
            .map(|unit| escape(unit.canonical()))
            .unwrap_or_else(|| "&mdash;".to_owned());
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td>{unit}</td></tr>",
            escape(parameter.id.as_str()),
            format_number(parameter.value)
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
}

fn trajectory_section(body: &mut String, world: &World, trajectory: &Trajectory) {
    let series: Vec<(String, Vec<f64>)> = world
        .state_ids()
        .filter_map(|id| {
            trajectory.values.get(id).map(|values| (id.as_str().to_owned(), values.clone()))
        })
        .collect();
    body.push_str("  <section>\n    <h2>Simulated trajectory</h2>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Default forward simulation over t &isin; [{}, {}] ({} samples).</p>",
        format_number(trajectory.time.first().copied().unwrap_or(0.0)),
        format_number(trajectory.time.last().copied().unwrap_or(0.0)),
        trajectory.samples()
    );
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&line_chart(&trajectory.time, &series, 720.0, 340.0));
    body.push_str("    </div>\n  </section>\n");
}

fn phase_section(body: &mut String, world: &World, trajectory: &Trajectory) {
    let states: Vec<_> = world.state_ids().collect();
    if states.len() < 2 {
        return;
    }
    let (x_id, y_id) = (states[0], states[1]);
    let (Some(x_values), Some(y_values)) =
        (trajectory.values.get(x_id), trajectory.values.get(y_id))
    else {
        return;
    };
    body.push_str("  <section>\n    <h2>Phase portrait</h2>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Trajectory in <span class=\"mono\">{}</span>&ndash;<span class=\"mono\">{}</span> space. <span style=\"color:{}\">&#9679;</span> start, <span style=\"color:#dc2626\">&#9679;</span> end.</p>",
        escape(x_id.as_str()),
        escape(y_id.as_str()),
        series_color(2)
    );
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&phase_portrait(x_id.as_str(), x_values, y_id.as_str(), y_values, 420.0, 420.0));
    body.push_str("    </div>\n  </section>\n");
}

fn document(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\" />\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n<title>{}</title>\n<style>\n{}\n</style>\n</head>\n<body>\n<main>\n{}</main>\n</body>\n</html>\n",
        escape(title),
        STYLE,
        body
    )
}

const STYLE: &str = "* { box-sizing: border-box; }
body { margin: 0; background: #f1f5f9; color: #0f172a;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; }
main { max-width: 820px; margin: 0 auto; padding: 32px 20px 64px; }
header { border-bottom: 2px solid #2563eb; padding-bottom: 12px; margin-bottom: 8px; }
h1 { font-size: 1.7rem; margin: 0 0 4px; }
h2 { font-size: 1.15rem; margin: 0 0 12px; color: #1e293b; }
.subtitle { margin: 0; color: #475569; font-size: 0.9rem; }
section { background: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px;
  padding: 20px 24px; margin-top: 20px; }
.equations { display: flex; flex-direction: column; gap: 8px; }
.equation { font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 0.98rem; background: #f8fafc; border-left: 3px solid #2563eb;
  padding: 8px 12px; border-radius: 4px; overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
th, td { text-align: left; padding: 6px 10px; border-bottom: 1px solid #e2e8f0; }
th { color: #475569; font-weight: 600; }
.mono { font-family: 'SFMono-Regular', Consolas, monospace; }
.muted { color: #64748b; font-size: 0.85rem; }
.chart { overflow-x: auto; }
svg { max-width: 100%; height: auto; border-radius: 6px; }
";

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_expr::Expr;
    use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn escapes_angle_brackets() {
        assert_eq!(escape("a<b>&\"c"), "a&lt;b&gt;&amp;&quot;c");
    }

    #[test]
    fn page_is_a_complete_document() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), 0.5)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
            )],
        )
        .unwrap();
        let trajectory = Trajectory {
            time: vec![0.0, 1.0],
            values: [(id("x"), vec![1.0, 0.37])].into_iter().collect(),
        };
        let html = page("Test", &world, &trajectory);
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("dx/dt = -x"));
        assert!(html.contains("<svg"));
        assert!(html.contains("</html>"));
    }
}
