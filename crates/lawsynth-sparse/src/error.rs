use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SparseError {
    EmptyProblem,
    RowLengthMismatch,
    NonFiniteValue,
    SingularSystem,
    InvalidConfig,
    InvalidGroups,
}

impl fmt::Display for SparseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProblem => {
                write!(formatter, "regression problem must have rows and features")
            }
            Self::RowLengthMismatch => write!(formatter, "feature rows and targets are misaligned"),
            Self::NonFiniteValue => write!(formatter, "regression values must be finite"),
            Self::SingularSystem => write!(formatter, "least-squares system is singular"),
            Self::InvalidConfig => write!(formatter, "sparse regression configuration is invalid"),
            Self::InvalidGroups => {
                write!(formatter, "feature groups must partition every feature exactly once")
            }
        }
    }
}

impl std::error::Error for SparseError {}
