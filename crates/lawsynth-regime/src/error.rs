use std::fmt;
#[derive(Debug, Clone, PartialEq)]
pub enum RegimeError {
    EmptySeries,
    NonFiniteObservation { index: usize },
    InvalidParameter(&'static str),
    InvalidSegment { start: usize, end: usize },
    InsufficientSamples { required: usize, actual: usize },
    DimensionMismatch { expected: usize, actual: usize },
    InvalidProbability,
    ImpossibleObservation { index: usize },
}
impl fmt::Display for RegimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySeries => write!(f, "series must not be empty"),
            Self::NonFiniteObservation { index } => write!(f, "observation {index} is not finite"),
            Self::InvalidParameter(n) => write!(f, "invalid parameter `{n}`"),
            Self::InvalidSegment { start, end } => write!(f, "invalid segment [{start}, {end})"),
            Self::InsufficientSamples { required, actual } => {
                write!(f, "requires at least {required} samples, got {actual}")
            }
            Self::DimensionMismatch { expected, actual } => {
                write!(f, "dimension mismatch: expected {expected}, got {actual}")
            }
            Self::InvalidProbability => write!(
                f,
                "probabilities must be finite, non-negative, and normalized"
            ),
            Self::ImpossibleObservation { index } => {
                write!(f, "no state can emit observation at index {index}")
            }
        }
    }
}
impl std::error::Error for RegimeError {}
pub type Result<T> = std::result::Result<T, RegimeError>;
