//! `lawsynth compose` — combine two worlds into one coupled system.
//!
//! The result is the union of both worlds' variables, parameters, and laws.
//! Identifiers that collide between the two systems are namespaced with a prefix
//! so both can coexist in a single validated World that simulates. Supplying an
//! explicit `--prefix-a` / `--prefix-b` namespaces *every* identifier of that
//! world; otherwise only the colliding identifiers are prefixed (defaults `a_`
//! and `b_`), keeping non-colliding names untouched.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use lawsynth_bundle::{read_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_world::{ContinuousLaw, Parameter, Variable, World};

use crate::worldops::{rename_world, world_identifiers};

/// Help text for `lawsynth compose`.
pub fn help() -> String {
    "lawsynth compose WORLD-A.lsworld WORLD-B.lsworld --output COMBINED.lsworld [--prefix-a A_] [--prefix-b B_]\n\n\
Combines two worlds into one coupled system: the union of their variables, \
parameters, and laws. Colliding identifiers are namespaced with a prefix so both \
systems coexist. --prefix-a / --prefix-b namespace every identifier of that \
world; without them, only colliding identifiers are prefixed (defaults a_ / b_). \
The result is a validated World that simulates."
        .to_owned()
}

struct ComposeArgs {
    path_a: String,
    path_b: String,
    output: String,
    prefix_a: Option<String>,
    prefix_b: Option<String>,
}

/// Runs the `compose` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let world_a = read_world(&args.path_a).map_err(|error| error.to_string())?;
    let world_b = read_world(&args.path_b).map_err(|error| error.to_string())?;

    let ids_a = world_identifiers(&world_a);
    let ids_b = world_identifiers(&world_b);
    let collisions: BTreeSet<Identifier> = ids_a.intersection(&ids_b).cloned().collect();

    let map_a = build_map(&ids_a, args.prefix_a.as_deref(), &collisions, "a_")?;
    let map_b = build_map(&ids_b, args.prefix_b.as_deref(), &collisions, "b_")?;

    let renamed_a = rename_world(&world_a, &map_a)?;
    let renamed_b = rename_world(&world_b, &map_b)?;

    let combined = merge(&renamed_a, &renamed_b)?;
    write_world(&args.output, &combined).map_err(|error| error.to_string())?;

    let renamed_count_a = map_a.iter().filter(|(old, new)| old != new).count();
    let renamed_count_b = map_b.iter().filter(|(old, new)| old != new).count();

    let mut out = String::new();
    let _ = writeln!(
        out,
        "composed world: {} variable(s), {} parameter(s), {} law(s)",
        combined.variables().len(),
        combined.parameters().len(),
        combined.laws().len()
    );
    let _ = writeln!(
        out,
        "  from A ({}): {} state(s), {} identifier(s) namespaced",
        args.path_a,
        renamed_a.state_ids().count(),
        renamed_count_a
    );
    let _ = writeln!(
        out,
        "  from B ({}): {} state(s), {} identifier(s) namespaced",
        args.path_b,
        renamed_b.state_ids().count(),
        renamed_count_b
    );
    if collisions.is_empty() {
        let _ = writeln!(out, "  no identifier collisions detected");
    } else {
        let names: Vec<&str> = collisions.iter().map(Identifier::as_str).collect();
        let _ = writeln!(out, "  resolved collisions: {}", names.join(", "));
    }
    let _ = writeln!(out, "wrote combined world: {}", args.output);
    Ok(out)
}

/// Builds the rename map for one world. With an explicit `prefix`, every
/// identifier is namespaced; otherwise only `collisions` get `default_prefix`.
fn build_map(
    ids: &BTreeSet<Identifier>,
    prefix: Option<&str>,
    collisions: &BTreeSet<Identifier>,
    default_prefix: &str,
) -> Result<BTreeMap<Identifier, Identifier>, String> {
    let mut map = BTreeMap::new();
    for id in ids {
        let renamed = match prefix {
            Some(prefix) => prefixed(prefix, id)?,
            None if collisions.contains(id) => prefixed(default_prefix, id)?,
            None => id.clone(),
        };
        map.insert(id.clone(), renamed);
    }
    Ok(map)
}

/// Prefixes an identifier, validating the result as a portable identifier.
fn prefixed(prefix: &str, id: &Identifier) -> Result<Identifier, String> {
    Identifier::new(format!("{prefix}{}", id.as_str())).map_err(|error| {
        format!("invalid identifier from prefix '{prefix}' + '{}': {error}", id.as_str())
    })
}

/// Unions the variables, parameters, and laws of two (already namespaced)
/// worlds into a single validated world.
fn merge(a: &World, b: &World) -> Result<World, String> {
    let variables: Vec<Variable> =
        a.variables().values().cloned().chain(b.variables().values().cloned()).collect();
    let parameters: Vec<Parameter> =
        a.parameters().values().cloned().chain(b.parameters().values().cloned()).collect();
    let laws: Vec<ContinuousLaw> =
        a.laws().values().cloned().chain(b.laws().values().cloned()).collect();
    World::new(variables, parameters, laws).map_err(|error| {
        format!("composed world failed validation ({error}); try explicit --prefix-a / --prefix-b")
    })
}

fn parse(arguments: &[String]) -> Result<ComposeArgs, String> {
    let (Some(path_a), Some(path_b)) = (arguments.first(), arguments.get(1)) else {
        return Err(help());
    };
    if path_a.starts_with('-') || path_b.starts_with('-') {
        return Err(help());
    }
    let mut output = None;
    let mut prefix_a = None;
    let mut prefix_b = None;
    let mut index = 2;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--output" => output = Some(value.clone()),
            "--prefix-a" => prefix_a = Some(value.clone()),
            "--prefix-b" => prefix_b = Some(value.clone()),
            _ => return Err(help()),
        }
        index += 2;
    }
    Ok(ComposeArgs {
        path_a: path_a.clone(),
        path_b: path_b.clone(),
        output: output.ok_or("missing required --output COMBINED.lsworld")?,
        prefix_a,
        prefix_b,
    })
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::Expr;
    use lawsynth_world::VariableRole;

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn decay_world(rate: f64) -> World {
        World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), rate)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("k")), Expr::symbol(id("x"))),
            )],
        )
        .unwrap()
    }

    #[test]
    fn namespaces_collisions_by_default() {
        let a = decay_world(-1.0);
        let b = decay_world(-2.0);
        let ids_a = world_identifiers(&a);
        let ids_b = world_identifiers(&b);
        let collisions: BTreeSet<Identifier> = ids_a.intersection(&ids_b).cloned().collect();
        let map_a = build_map(&ids_a, None, &collisions, "a_").unwrap();
        let map_b = build_map(&ids_b, None, &collisions, "b_").unwrap();
        let combined =
            merge(&rename_world(&a, &map_a).unwrap(), &rename_world(&b, &map_b).unwrap()).unwrap();
        assert_eq!(combined.state_ids().count(), 2);
        assert!(combined.variables().contains_key(&id("a_x")));
        assert!(combined.variables().contains_key(&id("b_x")));
        assert!(combined.parameters().contains_key(&id("a_k")));
        assert!(combined.parameters().contains_key(&id("b_k")));
    }

    #[test]
    fn explicit_prefix_namespaces_everything() {
        let ids = world_identifiers(&decay_world(-1.0));
        let map = build_map(&ids, Some("sys_"), &BTreeSet::new(), "a_").unwrap();
        assert_eq!(map[&id("x")], id("sys_x"));
        assert_eq!(map[&id("k")], id("sys_k"));
    }
}
