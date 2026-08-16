use std::fmt;

use lawsynth_core::Identifier;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
    DuplicateVariable(Identifier),
    DuplicateParameter(Identifier),
    ParameterConflictsWithVariable(Identifier),
    NonFiniteParameter(Identifier),
    DuplicateLaw(Identifier),
    StateVariableWithoutLaw(Identifier),
    LawTargetsNonState(Identifier),
    UnknownSymbol(Identifier),
    UnitMismatch(Identifier),
    Unit(lawsynth_units::UnitError),
}

impl fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateVariable(id) => {
                write!(formatter, "variable '{id}' is declared more than once")
            }
            Self::DuplicateParameter(id) => {
                write!(formatter, "parameter '{id}' is declared more than once")
            }
            Self::ParameterConflictsWithVariable(id) => {
                write!(
                    formatter,
                    "parameter '{id}' conflicts with a variable identifier"
                )
            }
            Self::NonFiniteParameter(id) => {
                write!(formatter, "parameter '{id}' must have a finite value")
            }
            Self::DuplicateLaw(id) => write!(formatter, "more than one law targets state '{id}'"),
            Self::StateVariableWithoutLaw(id) => {
                write!(formatter, "state variable '{id}' has no continuous law")
            }
            Self::LawTargetsNonState(id) => {
                write!(formatter, "law target '{id}' is not a state variable")
            }
            Self::UnknownSymbol(id) => {
                write!(
                    formatter,
                    "law expression references undeclared symbol '{id}'"
                )
            }
            Self::UnitMismatch(id) => {
                write!(
                    formatter,
                    "law for '{id}' has dimensions incompatible with its time derivative"
                )
            }
            Self::Unit(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for WorldError {}

impl From<lawsynth_units::UnitError> for WorldError {
    fn from(error: lawsynth_units::UnitError) -> Self {
        Self::Unit(error)
    }
}
