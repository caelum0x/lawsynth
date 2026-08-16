use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CausalError {
    EmptySeries,
    LengthMismatch { expected: usize, actual: usize },
    InsufficientSamples { required: usize, actual: usize },
    DuplicateVariable(String),
    UnknownVariable(String),
    SelfEdge(String),
    Cycle { from: String, to: String },
    NonMonotonicTime { index: usize },
    SingularDesign,
    InvalidParameter(&'static str),
}

impl fmt::Display for CausalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySeries => write!(f, "series must not be empty"),
            Self::LengthMismatch { expected, actual } => {
                write!(f, "length mismatch: expected {expected}, got {actual}")
            }
            Self::InsufficientSamples { required, actual } => {
                write!(f, "requires at least {required} samples, got {actual}")
            }
            Self::DuplicateVariable(v) => write!(f, "duplicate variable `{v}`"),
            Self::UnknownVariable(v) => write!(f, "unknown variable `{v}`"),
            Self::SelfEdge(v) => write!(f, "self edge for `{v}` is not allowed"),
            Self::Cycle { from, to } => write!(f, "edge `{from}` -> `{to}` would create a cycle"),
            Self::NonMonotonicTime { index } => {
                write!(f, "time is not strictly increasing at index {index}")
            }
            Self::SingularDesign => write!(f, "regression design is singular"),
            Self::InvalidParameter(name) => write!(f, "invalid parameter `{name}`"),
        }
    }
}

impl std::error::Error for CausalError {}
pub type Result<T> = std::result::Result<T, CausalError>;
