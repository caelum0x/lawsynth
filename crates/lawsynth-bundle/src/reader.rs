use std::{collections::BTreeMap, fs, path::Path};

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};
use lawsynth_units::Unit;
use lawsynth_world::{
    ContinuousLaw, DiscreteLaw, DiscreteWorld, Parameter, Variable, VariableRole, World,
};

use crate::{
    BundleError,
    checksum::sha256_hex,
    layout::read_archive,
    manifest::{self, CHECKSUM_PATH, MANIFEST_PATH, WORLD_PATH},
};

type DecodedComponents = (Vec<Variable>, Vec<Parameter>, Vec<(Identifier, Expr)>);

/// Opens a .lsworld archive and validates archive and SHA-256 checksums.
pub fn read_world(path: impl AsRef<Path>) -> Result<World, BundleError> {
    let bytes = read_world_bytes(path)?;
    decode_world(&bytes)
}

/// Opens a discrete-time .lsworld archive and validates archive and SHA-256 checksums.
pub fn read_discrete_world(path: impl AsRef<Path>) -> Result<DiscreteWorld, BundleError> {
    let bytes = read_world_bytes(path)?;
    decode_discrete_world(&bytes)
}

fn read_world_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>, BundleError> {
    let entries = read_archive(&fs::read(path)?)?;
    let manifest_contents = entries
        .get(MANIFEST_PATH)
        .ok_or(BundleError::MissingEntry(MANIFEST_PATH))?;
    if manifest_contents.as_slice() != manifest::contents() {
        return Err(BundleError::InvalidArchive("unsupported manifest"));
    }
    verify_checksums(&entries)?;
    Ok(entries
        .get(WORLD_PATH)
        .ok_or(BundleError::MissingEntry(WORLD_PATH))?
        .clone())
}

fn verify_checksums(entries: &BTreeMap<String, Vec<u8>>) -> Result<(), BundleError> {
    let checksum_file = entries
        .get(CHECKSUM_PATH)
        .ok_or(BundleError::MissingEntry(CHECKSUM_PATH))?;
    let checksum_file = std::str::from_utf8(checksum_file)
        .map_err(|_| BundleError::InvalidArchive("checksum file is not UTF-8"))?;
    let mut expected = BTreeMap::new();
    for line in checksum_file.lines() {
        let (hash, path) = line
            .split_once("  ")
            .ok_or(BundleError::InvalidArchive("invalid checksum line"))?;
        if hash.len() != 64 || path.is_empty() {
            return Err(BundleError::InvalidArchive("invalid checksum line"));
        }
        if expected.insert(path, hash).is_some() {
            return Err(BundleError::InvalidArchive("duplicate checksum entry"));
        }
    }
    for (path, content) in entries {
        if path == CHECKSUM_PATH {
            continue;
        }
        let expected_hash = expected
            .remove(path.as_str())
            .ok_or_else(|| BundleError::ChecksumMismatch(path.clone()))?;
        if expected_hash != sha256_hex(content) {
            return Err(BundleError::ChecksumMismatch(path.clone()));
        }
    }
    if !expected.is_empty() {
        return Err(BundleError::InvalidArchive(
            "checksum references a missing entry",
        ));
    }
    Ok(())
}

pub(crate) fn decode_world(bytes: &[u8]) -> Result<World, BundleError> {
    let (variables, parameters, laws) = decode_components(bytes, b"LSW1")?;
    World::new(
        variables,
        parameters,
        laws.into_iter()
            .map(|(target, expression)| ContinuousLaw::new(target, expression)),
    )
    .map_err(Into::into)
}

pub(crate) fn decode_discrete_world(bytes: &[u8]) -> Result<DiscreteWorld, BundleError> {
    let (variables, parameters, laws) = decode_components(bytes, b"LSD1")?;
    DiscreteWorld::new(
        variables,
        parameters,
        laws.into_iter()
            .map(|(target, expression)| DiscreteLaw::new(target, expression)),
    )
    .map_err(Into::into)
}

fn decode_components(
    bytes: &[u8],
    expected_magic: &[u8; 4],
) -> Result<DecodedComponents, BundleError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take_exact(4)? != expected_magic {
        return Err(BundleError::InvalidWorld(
            "bundle time semantics do not match requested world type",
        ));
    }
    let variables = (0..cursor.take_count()?)
        .map(|_| take_variable(&mut cursor))
        .collect::<Result<Vec<_>, _>>()?;
    let parameters = (0..cursor.take_count()?)
        .map(|_| take_parameter(&mut cursor))
        .collect::<Result<Vec<_>, _>>()?;
    let laws = (0..cursor.take_count()?)
        .map(|_| {
            let target = take_identifier(&mut cursor)?;
            let expression = take_expr(&mut cursor, 0)?;
            Ok((target, expression))
        })
        .collect::<Result<Vec<_>, BundleError>>()?;
    if !cursor.is_empty() {
        return Err(BundleError::InvalidWorld("trailing bytes"));
    }
    Ok((variables, parameters, laws))
}

fn take_variable(cursor: &mut Cursor<'_>) -> Result<Variable, BundleError> {
    let id = take_identifier(cursor)?;
    let role = match cursor.take_u8()? {
        0 => VariableRole::State,
        1 => VariableRole::Control,
        2 => VariableRole::Exogenous,
        3 => VariableRole::Observed,
        4 => VariableRole::Latent,
        5 => VariableRole::Derived,
        _ => return Err(BundleError::InvalidWorld("unknown variable role")),
    };
    let mut variable = Variable::new(id, role);
    if let Some(unit) = take_optional_string(cursor)? {
        variable = variable.with_unit(
            Unit::parse(&unit).map_err(|_| BundleError::InvalidWorld("invalid variable unit"))?,
        );
    }
    Ok(variable)
}

fn take_parameter(cursor: &mut Cursor<'_>) -> Result<Parameter, BundleError> {
    let id = take_identifier(cursor)?;
    let value = f64::from_le_bytes(cursor.take_exact(8)?.try_into().unwrap());
    if !value.is_finite() {
        return Err(BundleError::InvalidWorld("parameter must be finite"));
    }
    let mut parameter = Parameter::new(id, value);
    if let Some(unit) = take_optional_string(cursor)? {
        parameter = parameter.with_unit(
            Unit::parse(&unit).map_err(|_| BundleError::InvalidWorld("invalid parameter unit"))?,
        );
    }
    Ok(parameter)
}

fn take_expr(cursor: &mut Cursor<'_>, depth: u8) -> Result<Expr, BundleError> {
    if depth >= 128 {
        return Err(BundleError::InvalidWorld("expression depth exceeds 128"));
    }
    match cursor.take_u8()? {
        0 => {
            let value = f64::from_le_bytes(cursor.take_exact(8)?.try_into().unwrap());
            if value.is_finite() {
                Ok(Expr::constant(value))
            } else {
                Err(BundleError::InvalidWorld(
                    "expression constants must be finite",
                ))
            }
        }
        1 => Ok(Expr::symbol(take_identifier(cursor)?)),
        2 => {
            let operator = match cursor.take_u8()? {
                0 => UnaryOperator::Negate,
                1 => UnaryOperator::Exp,
                2 => UnaryOperator::Log,
                3 => UnaryOperator::Sin,
                4 => UnaryOperator::Cos,
                _ => return Err(BundleError::InvalidWorld("unknown unary operator")),
            };
            Ok(Expr::unary(operator, take_expr(cursor, depth + 1)?))
        }
        3 => {
            let operator = match cursor.take_u8()? {
                0 => BinaryOperator::Add,
                1 => BinaryOperator::Subtract,
                2 => BinaryOperator::Multiply,
                3 => BinaryOperator::Divide,
                4 => BinaryOperator::Power,
                _ => return Err(BundleError::InvalidWorld("unknown binary operator")),
            };
            Ok(Expr::binary(
                operator,
                take_expr(cursor, depth + 1)?,
                take_expr(cursor, depth + 1)?,
            ))
        }
        _ => Err(BundleError::InvalidWorld("unknown expression tag")),
    }
}

fn take_identifier(cursor: &mut Cursor<'_>) -> Result<Identifier, BundleError> {
    Identifier::new(take_string(cursor)?)
        .map_err(|_| BundleError::InvalidWorld("invalid identifier"))
}

fn take_optional_string(cursor: &mut Cursor<'_>) -> Result<Option<String>, BundleError> {
    match cursor.take_u8()? {
        0 => Ok(None),
        1 => take_string(cursor).map(Some),
        _ => Err(BundleError::InvalidWorld("invalid optional string marker")),
    }
}

fn take_string(cursor: &mut Cursor<'_>) -> Result<String, BundleError> {
    let length = usize::from(u16::from_le_bytes(
        cursor.take_exact(2)?.try_into().unwrap(),
    ));
    let bytes = cursor.take_exact(length)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| BundleError::InvalidWorld("string is not UTF-8"))
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take_exact(&mut self, length: usize) -> Result<&'a [u8], BundleError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(BundleError::InvalidWorld("value exceeds world bounds"))?;
        let value = self
            .bytes
            .get(self.position..end)
            .ok_or(BundleError::InvalidWorld("unexpected end of world"))?;
        self.position = end;
        Ok(value)
    }

    fn take_u8(&mut self) -> Result<u8, BundleError> {
        Ok(self.take_exact(1)?[0])
    }

    fn take_count(&mut self) -> Result<usize, BundleError> {
        usize::try_from(u32::from_le_bytes(self.take_exact(4)?.try_into().unwrap()))
            .map_err(|_| BundleError::InvalidWorld("item count is too large"))
    }

    fn is_empty(&self) -> bool {
        self.position == self.bytes.len()
    }
}
