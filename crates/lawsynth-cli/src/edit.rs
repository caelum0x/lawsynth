//! `lawsynth edit` — targeted, immutable edits to a world.
//!
//! Each operation produces a brand-new, re-validated World (never a mutation in
//! place), applied left-to-right in the order given on the command line:
//!
//! - `--rename OLD:NEW`      rename a variable/parameter across every law
//! - `--set-param NAME=VALUE` change a parameter's constant value
//! - `--drop-law TARGET`     remove a state and its law (must be unreferenced)
//! - `--scale-law TARGET=FACTOR` multiply a law's expression by a constant

use std::collections::BTreeMap;
use std::fmt::Write as _;

use lawsynth_bundle::{read_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::World;

use crate::worldops::{decompose, recompose, rename_world, world_identifiers};

/// A single targeted edit.
enum EditOp {
    Rename { old: Identifier, new: Identifier },
    SetParam { name: Identifier, value: f64 },
    DropLaw { target: Identifier },
    ScaleLaw { target: Identifier, factor: f64 },
}

/// Help text for `lawsynth edit`.
pub fn help() -> String {
    "lawsynth edit WORLD.lsworld --output EDITED.lsworld [--rename OLD:NEW] [--set-param NAME=VALUE] [--drop-law TARGET] [--scale-law TARGET=FACTOR]\n\n\
Applies targeted edits, in order, each producing a new validated World:\n  \
--rename OLD:NEW          rename a variable/parameter consistently across all laws\n  \
--set-param NAME=VALUE    set a parameter's value\n  \
--drop-law TARGET         remove a state variable and its law (must be unreferenced)\n  \
--scale-law TARGET=FACTOR multiply a law's expression by a constant factor\n\n\
The edited world is re-validated and written to --output."
        .to_owned()
}

/// Runs the `edit` command.
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
    let (output, ops) = parse(&arguments[1..])?;
    if ops.is_empty() {
        return Err("no edit operations given; nothing to do".to_owned());
    }

    let original = read_world(bundle).map_err(|error| error.to_string())?;
    let mut world = original.clone();
    let mut applied = Vec::new();
    for op in &ops {
        let (next, description) = apply(&world, op)?;
        world = next;
        applied.push(description);
    }

    write_world(&output, &world).map_err(|error| error.to_string())?;

    let mut out = String::new();
    let _ = writeln!(out, "edited {bundle}");
    for description in &applied {
        let _ = writeln!(out, "  - {description}");
    }
    let _ = writeln!(
        out,
        "result: {} variable(s), {} parameter(s), {} law(s)  (validated)",
        world.variables().len(),
        world.parameters().len(),
        world.laws().len()
    );
    let _ = writeln!(out, "wrote edited world: {output}");
    Ok(out)
}

/// Applies a single edit, returning the new world and a human-readable summary.
fn apply(world: &World, op: &EditOp) -> Result<(World, String), String> {
    match op {
        EditOp::Rename { old, new } => {
            if !world_identifiers(world).contains(old) {
                return Err(format!(
                    "cannot rename '{}': no such variable or parameter",
                    old.as_str()
                ));
            }
            let map = BTreeMap::from([(old.clone(), new.clone())]);
            let renamed = rename_world(world, &map)?;
            Ok((renamed, format!("renamed {} -> {}", old.as_str(), new.as_str())))
        }
        EditOp::SetParam { name, value } => {
            let (variables, mut parameters, laws) = decompose(world);
            let parameter = parameters
                .iter_mut()
                .find(|parameter| &parameter.id == name)
                .ok_or_else(|| format!("cannot set '{}': no such parameter", name.as_str()))?;
            let previous = parameter.value;
            parameter.value = *value;
            let next = recompose(variables, parameters, laws)?;
            Ok((next, format!("set-param {} = {} (was {})", name.as_str(), value, previous)))
        }
        EditOp::DropLaw { target } => {
            let (variables, parameters, laws) = decompose(world);
            if !laws.iter().any(|law| &law.target == target) {
                return Err(format!("cannot drop law '{}': no such law", target.as_str()));
            }
            // Removing a state's law requires removing the state itself, or the
            // world would have a state with no law. `recompose` then rejects the
            // edit if any surviving law still reads the dropped symbol.
            let variables =
                variables.into_iter().filter(|variable| &variable.id != target).collect();
            let laws = laws.into_iter().filter(|law| &law.target != target).collect();
            let next = recompose(variables, parameters, laws)
                .map_err(|error| format!("cannot drop law '{}': {error}", target.as_str()))?;
            Ok((next, format!("dropped law and state {}", target.as_str())))
        }
        EditOp::ScaleLaw { target, factor } => {
            let (variables, parameters, mut laws) = decompose(world);
            let law = laws
                .iter_mut()
                .find(|law| &law.target == target)
                .ok_or_else(|| format!("cannot scale law '{}': no such law", target.as_str()))?;
            law.expression = Expr::product(Expr::constant(*factor), law.expression.clone());
            let next = recompose(variables, parameters, laws)?;
            Ok((next, format!("scaled law d{}/dt by {}", target.as_str(), factor)))
        }
    }
}

fn parse(arguments: &[String]) -> Result<(String, Vec<EditOp>), String> {
    let mut output = None;
    let mut ops = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--output" => output = Some(value.clone()),
            "--rename" => {
                let (old, new) =
                    value.split_once(':').ok_or_else(|| "expected --rename OLD:NEW".to_owned())?;
                ops.push(EditOp::Rename { old: identifier(old)?, new: identifier(new)? });
            }
            "--set-param" => {
                let (name, raw) = value
                    .split_once('=')
                    .ok_or_else(|| "expected --set-param NAME=VALUE".to_owned())?;
                ops.push(EditOp::SetParam { name: identifier(name)?, value: number(raw)? });
            }
            "--drop-law" => ops.push(EditOp::DropLaw { target: identifier(value)? }),
            "--scale-law" => {
                let (target, raw) = value
                    .split_once('=')
                    .ok_or_else(|| "expected --scale-law TARGET=FACTOR".to_owned())?;
                ops.push(EditOp::ScaleLaw { target: identifier(target)?, factor: number(raw)? });
            }
            _ => return Err(help()),
        }
        index += 2;
    }
    Ok((output.ok_or("missing required --output EDITED.lsworld")?, ops))
}

fn identifier(value: &str) -> Result<Identifier, String> {
    Identifier::new(value.trim()).map_err(|error| error.to_string())
}

fn number(value: &str) -> Result<f64, String> {
    let number: f64 = value.trim().parse().map_err(|_| format!("invalid number '{value}'"))?;
    if number.is_finite() { Ok(number) } else { Err(format!("number '{value}' must be finite")) }
}

#[cfg(test)]
mod tests {
    use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn world() -> World {
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            [Parameter::new(id("k"), 1.5)],
            [
                ContinuousLaw::new(
                    id("x"),
                    Expr::product(Expr::symbol(id("k")), Expr::symbol(id("x"))),
                ),
                ContinuousLaw::new(id("y"), Expr::symbol(id("y"))),
            ],
        )
        .unwrap()
    }

    #[test]
    fn renames_parameter_across_laws() {
        let map = BTreeMap::from([(id("k"), id("rate"))]);
        let renamed = rename_world(&world(), &map).unwrap();
        assert!(renamed.parameters().contains_key(&id("rate")));
        assert_eq!(
            renamed.laws()[&id("x")].expression,
            Expr::product(Expr::symbol(id("rate")), Expr::symbol(id("x")))
        );
    }

    #[test]
    fn sets_parameter_value() {
        let (next, description) =
            apply(&world(), &EditOp::SetParam { name: id("k"), value: 3.0 }).unwrap();
        assert_eq!(next.parameters()[&id("k")].value, 3.0);
        assert!(description.contains("was 1.5"));
    }

    #[test]
    fn scaling_a_law_wraps_the_expression() {
        let (next, _) =
            apply(&world(), &EditOp::ScaleLaw { target: id("y"), factor: 2.0 }).unwrap();
        assert_eq!(
            next.laws()[&id("y")].expression,
            Expr::product(Expr::constant(2.0), Expr::symbol(id("y")))
        );
    }

    #[test]
    fn dropping_a_referenced_law_is_rejected() {
        // y's law is independent, so dropping y is fine; dropping x is fine too
        // here because nothing else references it. Build a coupled world instead.
        let coupled = World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            [],
            [
                ContinuousLaw::new(id("x"), Expr::symbol(id("y"))),
                ContinuousLaw::new(id("y"), Expr::symbol(id("x"))),
            ],
        )
        .unwrap();
        let result = apply(&coupled, &EditOp::DropLaw { target: id("x") });
        assert!(result.is_err(), "dropping a referenced state should fail");
    }

    #[test]
    fn dropping_an_unreferenced_law_succeeds() {
        let (next, _) = apply(&world(), &EditOp::DropLaw { target: id("y") }).unwrap();
        assert!(!next.variables().contains_key(&id("y")));
        assert_eq!(next.state_ids().count(), 1);
    }
}
