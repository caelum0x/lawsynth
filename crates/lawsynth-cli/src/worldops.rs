//! Small, reusable immutable transforms on continuous World IR.
//!
//! These helpers rebuild a fresh, re-validated [`World`] rather than mutating an
//! existing one, so `compose` and `edit` always emit worlds that pass the same
//! construction-time validation the rest of the toolchain relies on.

use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{ContinuousLaw, Parameter, Variable, World};

/// Returns the mapped identifier, or a clone of the original when absent.
pub fn map_identifier(id: &Identifier, map: &BTreeMap<Identifier, Identifier>) -> Identifier {
    map.get(id).cloned().unwrap_or_else(|| id.clone())
}

/// Rewrites every symbol in `expression` through `map`, leaving constants and
/// operators untouched. Identifiers not present in `map` are preserved.
pub fn rename_symbols(expression: &Expr, map: &BTreeMap<Identifier, Identifier>) -> Expr {
    match expression {
        Expr::Constant(_) => expression.clone(),
        Expr::Symbol(id) => Expr::symbol(map_identifier(id, map)),
        Expr::Unary { operator, operand } => Expr::unary(*operator, rename_symbols(operand, map)),
        Expr::Binary { operator, left, right } => {
            Expr::binary(*operator, rename_symbols(left, map), rename_symbols(right, map))
        }
    }
}

/// Clones a variable with a remapped identifier, preserving role and unit.
fn remap_variable(variable: &Variable, map: &BTreeMap<Identifier, Identifier>) -> Variable {
    Variable {
        id: map_identifier(&variable.id, map),
        role: variable.role,
        unit: variable.unit.clone(),
    }
}

/// Clones a parameter with a remapped identifier, preserving value and unit.
fn remap_parameter(parameter: &Parameter, map: &BTreeMap<Identifier, Identifier>) -> Parameter {
    Parameter {
        id: map_identifier(&parameter.id, map),
        value: parameter.value,
        unit: parameter.unit.clone(),
    }
}

/// Returns a new, re-validated world with every identifier remapped through
/// `map` (variables, parameters, law targets, and expression symbols alike).
pub fn rename_world(
    world: &World,
    map: &BTreeMap<Identifier, Identifier>,
) -> Result<World, String> {
    let variables: Vec<Variable> =
        world.variables().values().map(|variable| remap_variable(variable, map)).collect();
    let parameters: Vec<Parameter> =
        world.parameters().values().map(|parameter| remap_parameter(parameter, map)).collect();
    let laws: Vec<ContinuousLaw> = world
        .laws()
        .values()
        .map(|law| {
            ContinuousLaw::new(
                map_identifier(&law.target, map),
                rename_symbols(&law.expression, map),
            )
        })
        .collect();
    World::new(variables, parameters, laws).map_err(|error| error.to_string())
}

/// Collects the full identifier namespace of a world (variables and parameters).
pub fn world_identifiers(world: &World) -> std::collections::BTreeSet<Identifier> {
    world.variables().keys().chain(world.parameters().keys()).cloned().collect()
}

/// Decomposes a world into owned components for reconstruction after an edit.
pub fn decompose(world: &World) -> (Vec<Variable>, Vec<Parameter>, Vec<ContinuousLaw>) {
    (
        world.variables().values().cloned().collect(),
        world.parameters().values().cloned().collect(),
        world.laws().values().cloned().collect(),
    )
}

/// Rebuilds and re-validates a world from owned components.
pub fn recompose(
    variables: Vec<Variable>,
    parameters: Vec<Parameter>,
    laws: Vec<ContinuousLaw>,
) -> Result<World, String> {
    World::new(variables, parameters, laws).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use lawsynth_world::VariableRole;

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn renames_symbols_consistently() {
        let map = BTreeMap::from([(id("x"), id("a_x"))]);
        let expression = Expr::product(Expr::symbol(id("k")), Expr::symbol(id("x")));
        let renamed = rename_symbols(&expression, &map);
        assert_eq!(renamed, Expr::product(Expr::symbol(id("k")), Expr::symbol(id("a_x"))));
    }

    #[test]
    fn rename_world_rewrites_targets_and_symbols() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), 2.0)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("k")), Expr::symbol(id("x"))),
            )],
        )
        .unwrap();
        let map = BTreeMap::from([(id("x"), id("y")), (id("k"), id("rate"))]);
        let renamed = rename_world(&world, &map).unwrap();
        assert!(renamed.variables().contains_key(&id("y")));
        assert!(renamed.parameters().contains_key(&id("rate")));
        assert_eq!(
            renamed.laws()[&id("y")].expression,
            Expr::product(Expr::symbol(id("rate")), Expr::symbol(id("y")))
        );
    }
}
