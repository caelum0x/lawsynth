use std::fmt;

use lawsynth_core::Identifier;

#[derive(Clone, Debug, PartialEq)]
pub enum EvaluationError {
    UnknownSymbol(Identifier),
    DivisionByZero,
    DomainError { operation: &'static str, input: f64 },
    NonFiniteResult,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSymbol(symbol) => {
                write!(formatter, "no value was supplied for symbol '{symbol}'")
            }
            Self::DivisionByZero => write!(formatter, "division by zero"),
            Self::DomainError { operation, input } => {
                write!(formatter, "{operation} is undefined for {input}")
            }
            Self::NonFiniteResult => {
                write!(formatter, "expression evaluated to a non-finite value")
            }
        }
    }
}

impl std::error::Error for EvaluationError {}
