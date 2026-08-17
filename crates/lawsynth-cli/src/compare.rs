//! `lawsynth compare` — structural and parameter diff of two worlds.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::fs;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_report::{format_number, render_comparison, render_expression};
use lawsynth_world::{Variable, VariableRole, World};

/// Help text for `lawsynth compare`.
pub fn help() -> String {
    "lawsynth compare WORLD-A.lsworld WORLD-B.lsworld [--json] [--html FILE]\n\n\
Diffs two worlds: variables and parameters added/removed/changed, per-law \
structural and parameter differences, and a complexity comparison. With --html, \
writes a self-contained side-by-side HTML diff instead of text."
        .to_owned()
}

/// Runs the `compare` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if arguments.first().map(String::as_str) == Some("--help")
        || arguments.first().map(String::as_str) == Some("-h")
    {
        return Ok(help());
    }
    let (Some(path_a), Some(path_b)) = (arguments.first(), arguments.get(1)) else {
        return Err(help());
    };
    let mut json = false;
    let mut html: Option<String> = None;
    let mut index = 2;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--json" => {
                json = true;
                index += 1;
            }
            "--html" => {
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --html".to_owned())?;
                html = Some(value.clone());
                index += 2;
            }
            _ => return Err(help()),
        }
    }
    let world_a = read_world(path_a).map_err(|error| error.to_string())?;
    let world_b = read_world(path_b).map_err(|error| error.to_string())?;

    if let Some(html_path) = html {
        let document = render_comparison(&world_a, path_a, &world_b, path_b);
        fs::write(&html_path, &document)
            .map_err(|error| format!("failed to write {html_path}: {error}"))?;
        return Ok(format!("wrote comparison: {html_path} ({} bytes)\n", document.len()));
    }

    let diff = WorldDiff::compute(&world_a, &world_b);
    if json { Ok(diff.to_json(path_a, path_b)) } else { Ok(diff.to_text(path_a, path_b)) }
}

struct WorldDiff {
    variables_added: Vec<String>,
    variables_removed: Vec<String>,
    variables_changed: Vec<String>,
    parameters_added: Vec<String>,
    parameters_removed: Vec<String>,
    parameters_changed: Vec<String>,
    laws_added: Vec<String>,
    laws_removed: Vec<String>,
    laws_changed: Vec<String>,
    complexity_a: usize,
    complexity_b: usize,
}

impl WorldDiff {
    fn compute(a: &World, b: &World) -> Self {
        let variable_ids = union_ids(a.variables().keys(), b.variables().keys());
        let mut variables_added = Vec::new();
        let mut variables_removed = Vec::new();
        let mut variables_changed = Vec::new();
        for id in &variable_ids {
            match (a.variables().get(id), b.variables().get(id)) {
                (None, Some(variable)) => variables_added.push(format!(
                    "{} ({})",
                    id.as_str(),
                    describe_variable(variable)
                )),
                (Some(variable), None) => variables_removed.push(format!(
                    "{} ({})",
                    id.as_str(),
                    describe_variable(variable)
                )),
                (Some(left), Some(right)) if left != right => variables_changed.push(format!(
                    "{}: {} -> {}",
                    id.as_str(),
                    describe_variable(left),
                    describe_variable(right)
                )),
                _ => {}
            }
        }

        let parameter_ids = union_ids(a.parameters().keys(), b.parameters().keys());
        let mut parameters_added = Vec::new();
        let mut parameters_removed = Vec::new();
        let mut parameters_changed = Vec::new();
        for id in &parameter_ids {
            match (a.parameters().get(id), b.parameters().get(id)) {
                (None, Some(parameter)) => parameters_added.push(format!(
                    "{} = {}",
                    id.as_str(),
                    format_number(parameter.value)
                )),
                (Some(parameter), None) => parameters_removed.push(format!(
                    "{} = {}",
                    id.as_str(),
                    format_number(parameter.value)
                )),
                (Some(left), Some(right))
                    if left.value != right.value || left.unit != right.unit =>
                {
                    parameters_changed.push(format!(
                        "{}: {} -> {}",
                        id.as_str(),
                        format_number(left.value),
                        format_number(right.value)
                    ))
                }
                _ => {}
            }
        }

        let law_ids = union_ids(a.laws().keys(), b.laws().keys());
        let mut laws_added = Vec::new();
        let mut laws_removed = Vec::new();
        let mut laws_changed = Vec::new();
        for id in &law_ids {
            match (a.laws().get(id), b.laws().get(id)) {
                (None, Some(law)) => laws_added.push(format!(
                    "d{}/dt = {}",
                    id.as_str(),
                    render_expression(&law.expression)
                )),
                (Some(law), None) => laws_removed.push(format!(
                    "d{}/dt = {}",
                    id.as_str(),
                    render_expression(&law.expression)
                )),
                (Some(left), Some(right))
                    if left.expression.to_canonical_string()
                        != right.expression.to_canonical_string() =>
                {
                    laws_changed.push(format!(
                        "{}: {} ({} nodes) -> {} ({} nodes)",
                        id.as_str(),
                        render_expression(&left.expression),
                        complexity(&left.expression),
                        render_expression(&right.expression),
                        complexity(&right.expression)
                    ))
                }
                _ => {}
            }
        }

        Self {
            variables_added,
            variables_removed,
            variables_changed,
            parameters_added,
            parameters_removed,
            parameters_changed,
            laws_added,
            laws_removed,
            laws_changed,
            complexity_a: total_complexity(a),
            complexity_b: total_complexity(b),
        }
    }

    fn to_text(&self, path_a: &str, path_b: &str) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "Comparing worlds");
        let _ = writeln!(out, "  A: {path_a}");
        let _ = writeln!(out, "  B: {path_b}");
        out.push('\n');

        section(
            &mut out,
            "Variables",
            &[
                ("added", &self.variables_added),
                ("removed", &self.variables_removed),
                ("changed", &self.variables_changed),
            ],
        );
        section(
            &mut out,
            "Parameters",
            &[
                ("added", &self.parameters_added),
                ("removed", &self.parameters_removed),
                ("changed", &self.parameters_changed),
            ],
        );
        section(
            &mut out,
            "Laws",
            &[
                ("added", &self.laws_added),
                ("removed", &self.laws_removed),
                ("changed", &self.laws_changed),
            ],
        );

        let _ = writeln!(out, "Complexity");
        let delta = self.complexity_b as isize - self.complexity_a as isize;
        let _ = writeln!(
            out,
            "  A: {} node(s)   B: {} node(s)   delta: {:+}",
            self.complexity_a, self.complexity_b, delta
        );
        if !self.has_differences() {
            out.push('\n');
            let _ = writeln!(out, "Worlds are structurally identical.");
        }
        out
    }

    fn to_json(&self, path_a: &str, path_b: &str) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        let _ = writeln!(out, "  \"world_a\": {},", json_string(path_a));
        let _ = writeln!(out, "  \"world_b\": {},", json_string(path_b));
        let _ = writeln!(out, "  \"variables\": {{");
        let _ = writeln!(out, "    \"added\": {},", json_array(&self.variables_added));
        let _ = writeln!(out, "    \"removed\": {},", json_array(&self.variables_removed));
        let _ = writeln!(out, "    \"changed\": {}", json_array(&self.variables_changed));
        let _ = writeln!(out, "  }},");
        let _ = writeln!(out, "  \"parameters\": {{");
        let _ = writeln!(out, "    \"added\": {},", json_array(&self.parameters_added));
        let _ = writeln!(out, "    \"removed\": {},", json_array(&self.parameters_removed));
        let _ = writeln!(out, "    \"changed\": {}", json_array(&self.parameters_changed));
        let _ = writeln!(out, "  }},");
        let _ = writeln!(out, "  \"laws\": {{");
        let _ = writeln!(out, "    \"added\": {},", json_array(&self.laws_added));
        let _ = writeln!(out, "    \"removed\": {},", json_array(&self.laws_removed));
        let _ = writeln!(out, "    \"changed\": {}", json_array(&self.laws_changed));
        let _ = writeln!(out, "  }},");
        let _ = writeln!(out, "  \"complexity\": {{");
        let _ = writeln!(out, "    \"a\": {},", self.complexity_a);
        let _ = writeln!(out, "    \"b\": {},", self.complexity_b);
        let _ = writeln!(
            out,
            "    \"delta\": {}",
            self.complexity_b as isize - self.complexity_a as isize
        );
        let _ = writeln!(out, "  }}");
        out.push_str("}\n");
        out
    }

    fn has_differences(&self) -> bool {
        !(self.variables_added.is_empty()
            && self.variables_removed.is_empty()
            && self.variables_changed.is_empty()
            && self.parameters_added.is_empty()
            && self.parameters_removed.is_empty()
            && self.parameters_changed.is_empty()
            && self.laws_added.is_empty()
            && self.laws_removed.is_empty()
            && self.laws_changed.is_empty())
    }
}

fn section(out: &mut String, title: &str, groups: &[(&str, &Vec<String>)]) {
    let _ = writeln!(out, "{title}");
    let mut any = false;
    for (label, entries) in groups {
        for entry in *entries {
            let _ = writeln!(out, "  [{label}] {entry}");
            any = true;
        }
    }
    if !any {
        let _ = writeln!(out, "  (no changes)");
    }
    out.push('\n');
}

fn describe_variable(variable: &Variable) -> String {
    let role = match variable.role {
        VariableRole::State => "state",
        VariableRole::Control => "control",
        VariableRole::Exogenous => "exogenous",
        VariableRole::Observed => "observed",
        VariableRole::Latent => "latent",
        VariableRole::Derived => "derived",
    };
    match &variable.unit {
        Some(unit) => format!("{role}, {}", unit.canonical()),
        None => role.to_owned(),
    }
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

fn json_array(entries: &[String]) -> String {
    if entries.is_empty() {
        return "[]".to_owned();
    }
    let items: Vec<String> = entries.iter().map(|entry| json_string(entry)).collect();
    format!("[{}]", items.join(", "))
}

#[cfg(test)]
mod tests {
    use lawsynth_world::{ContinuousLaw, Parameter};

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
    fn detects_parameter_change() {
        let diff = WorldDiff::compute(&world(1.0), &world(2.0));
        assert_eq!(diff.parameters_changed.len(), 1);
        assert!(diff.parameters_changed[0].contains("1 -> 2"));
        assert!(diff.has_differences());
    }

    #[test]
    fn identical_worlds_have_no_differences() {
        let diff = WorldDiff::compute(&world(1.0), &world(1.0));
        assert!(!diff.has_differences());
        assert!(diff.to_text("a", "b").contains("structurally identical"));
    }
}
