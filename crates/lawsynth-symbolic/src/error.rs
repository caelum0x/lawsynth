use std::fmt;

/// Failures while calibrating a symbolic candidate against observations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolicError {
    EmptyInput,
    LengthMismatch,
    Evaluation(String),
    Optimization(String),
}

impl fmt::Display for SymbolicError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(formatter, "candidate calibration needs observations"),
            Self::LengthMismatch => {
                write!(formatter, "contexts and targets must have equal lengths")
            }
            Self::Evaluation(error) => write!(formatter, "candidate evaluation failed: {error}"),
            Self::Optimization(error) => write!(formatter, "constant optimization failed: {error}"),
        }
    }
}

impl std::error::Error for SymbolicError {}
