use std::{collections::BTreeMap, fs, path::Path};

use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};
use lawsynth_units::Unit;
use lawsynth_world::{DiscreteWorld, Parameter, Variable, VariableRole, World};

use crate::{
    BundleError,
    checksum::sha256_hex,
    layout::write_archive,
    manifest::{self, CHECKSUM_PATH, MANIFEST_PATH, WORLD_PATH},
};

/// Writes a deterministic stored-ZIP .lsworld archive.
pub fn write_world(path: impl AsRef<Path>, world: &World) -> Result<(), BundleError> {
    write_encoded_world(path, encode_world(world)?)
}

/// Writes a deterministic stored-ZIP .lsworld archive for a discrete-time world.
pub fn write_discrete_world(
    path: impl AsRef<Path>,
    world: &DiscreteWorld,
) -> Result<(), BundleError> {
    write_encoded_world(path, encode_discrete_world(world)?)
}

fn write_encoded_world(path: impl AsRef<Path>, world_bytes: Vec<u8>) -> Result<(), BundleError> {
    let mut entries = BTreeMap::from([
        (MANIFEST_PATH.to_owned(), manifest::contents().to_vec()),
        (WORLD_PATH.to_owned(), world_bytes),
    ]);
    let checksums = entries
        .iter()
        .map(|(entry_path, content)| format!("{}  {entry_path}\n", sha256_hex(content)))
        .collect::<String>();
    entries.insert(CHECKSUM_PATH.to_owned(), checksums.into_bytes());
    fs::write(path, write_archive(&entries)?)?;
    Ok(())
}

pub(crate) fn encode_world(world: &World) -> Result<Vec<u8>, BundleError> {
    encode_components(
        b"LSW1",
        world.variables().values(),
        world.parameters().values(),
        world
            .laws()
            .values()
            .map(|law| (&law.target, &law.expression)),
    )
}

pub(crate) fn encode_discrete_world(world: &DiscreteWorld) -> Result<Vec<u8>, BundleError> {
    encode_components(
        b"LSD1",
        world.variables().values(),
        world.parameters().values(),
        world
            .laws()
            .values()
            .map(|law| (&law.target, &law.expression)),
    )
}

fn encode_components<'a>(
    magic: &[u8; 4],
    variables: impl ExactSizeIterator<Item = &'a Variable>,
    parameters: impl ExactSizeIterator<Item = &'a Parameter>,
    laws: impl ExactSizeIterator<Item = (&'a lawsynth_core::Identifier, &'a Expr)>,
) -> Result<Vec<u8>, BundleError> {
    let mut output = magic.to_vec();
    put_count(&mut output, variables.len())?;
    for variable in variables {
        put_variable(&mut output, variable)?;
    }
    put_count(&mut output, parameters.len())?;
    for parameter in parameters {
        put_parameter(&mut output, parameter)?;
    }
    put_count(&mut output, laws.len())?;
    for (target, expression) in laws {
        put_string(&mut output, target.as_str())?;
        put_expr(&mut output, expression, 0)?;
    }
    Ok(output)
}

fn put_variable(output: &mut Vec<u8>, variable: &Variable) -> Result<(), BundleError> {
    put_string(output, variable.id.as_str())?;
    output.push(match variable.role {
        VariableRole::State => 0,
        VariableRole::Control => 1,
        VariableRole::Exogenous => 2,
        VariableRole::Observed => 3,
        VariableRole::Latent => 4,
        VariableRole::Derived => 5,
    });
    put_optional_string(output, variable.unit.as_ref().map(Unit::canonical))
}

fn put_parameter(output: &mut Vec<u8>, parameter: &Parameter) -> Result<(), BundleError> {
    put_string(output, parameter.id.as_str())?;
    output.extend(parameter.value.to_le_bytes());
    put_optional_string(output, parameter.unit.as_ref().map(Unit::canonical))
}

fn put_expr(output: &mut Vec<u8>, expression: &Expr, depth: u8) -> Result<(), BundleError> {
    if depth >= 128 {
        return Err(BundleError::InvalidWorld("expression depth exceeds 128"));
    }
    match expression {
        Expr::Constant(value) if value.is_finite() => {
            output.push(0);
            output.extend(value.to_le_bytes());
        }
        Expr::Constant(_) => {
            return Err(BundleError::InvalidWorld(
                "expression constants must be finite",
            ));
        }
        Expr::Symbol(id) => {
            output.push(1);
            put_string(output, id.as_str())?;
        }
        Expr::Unary { operator, operand } => {
            output.push(2);
            output.push(match operator {
                UnaryOperator::Negate => 0,
                UnaryOperator::Exp => 1,
                UnaryOperator::Log => 2,
                UnaryOperator::Sin => 3,
                UnaryOperator::Cos => 4,
            });
            put_expr(output, operand, depth + 1)?;
        }
        Expr::Binary {
            operator,
            left,
            right,
        } => {
            output.push(3);
            output.push(match operator {
                BinaryOperator::Add => 0,
                BinaryOperator::Subtract => 1,
                BinaryOperator::Multiply => 2,
                BinaryOperator::Divide => 3,
                BinaryOperator::Power => 4,
            });
            put_expr(output, left, depth + 1)?;
            put_expr(output, right, depth + 1)?;
        }
    }
    Ok(())
}

fn put_optional_string(output: &mut Vec<u8>, value: Option<&str>) -> Result<(), BundleError> {
    match value {
        Some(value) => {
            output.push(1);
            put_string(output, value)
        }
        None => {
            output.push(0);
            Ok(())
        }
    }
}

fn put_string(output: &mut Vec<u8>, value: &str) -> Result<(), BundleError> {
    let length = u16::try_from(value.len())
        .map_err(|_| BundleError::InvalidWorld("string exceeds 65,535 bytes"))?;
    output.extend(length.to_le_bytes());
    output.extend(value.as_bytes());
    Ok(())
}

fn put_count(output: &mut Vec<u8>, count: usize) -> Result<(), BundleError> {
    let count =
        u32::try_from(count).map_err(|_| BundleError::InvalidWorld("too many world items"))?;
    output.extend(count.to_le_bytes());
    Ok(())
}
