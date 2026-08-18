use std::fmt;

/// Errors returned when an uncertainty calculation would be undefined.
#[derive(Clone, Debug, PartialEq)]
pub enum UncertaintyError {
    EmptyInput,
    TooFewSamples { minimum: usize, actual: usize },
    NonFiniteValue,
    DimensionMismatch { expected: usize, actual: usize },
    InvalidConfidence(f64),
    InvalidBootstrapConfig,
    InvalidPropagationConfig,
    SingularCovariance,
    NonPositiveVariance,
    InsufficientResamples,
    FitFailure(String),
}

impl fmt::Display for UncertaintyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "uncertainty calculations require at least one value"),
            Self::TooFewSamples { minimum, actual } => {
                write!(f, "requires at least {minimum} samples, got {actual}")
            }
            Self::NonFiniteValue => write!(f, "uncertainty inputs must be finite"),
            Self::DimensionMismatch { expected, actual } => {
                write!(f, "dimension mismatch: expected {expected}, got {actual}")
            }
            Self::InvalidConfidence(value) => {
                write!(f, "confidence must be finite and in (0, 1), got {value}")
            }
            Self::InvalidBootstrapConfig => write!(f, "bootstrap replicates must be positive"),
            Self::InvalidPropagationConfig => {
                write!(f, "propagation sample count must be positive")
            }
            Self::SingularCovariance => write!(f, "covariance matrix is singular"),
            Self::NonPositiveVariance => write!(f, "variance must be positive"),
            Self::InsufficientResamples => {
                write!(f, "insufficient resamples for requested confidence interval")
            }
            Self::FitFailure(reason) => {
                write!(f, "sparse refit failed during bootstrap: {reason}")
            }
        }
    }
}

impl std::error::Error for UncertaintyError {}
