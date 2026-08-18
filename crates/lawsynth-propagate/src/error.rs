//! Typed errors for forecast-uncertainty propagation.

use std::fmt;

use lawsynth_sensitivity::SensitivityError;

/// Every failure mode of the propagation layer, reported explicitly.
///
/// The propagator never fabricates a band to paper over an ill-posed input: a
/// mis-shaped covariance, an indefinite covariance, a degenerate sample count,
/// or an upstream integration failure each surface as a distinct typed error.
#[derive(Clone, Debug, PartialEq)]
pub enum PropagateError {
    /// The forward-sensitivity integration (or the Monte-Carlo re-simulation)
    /// failed; the underlying [`SensitivityError`] is preserved verbatim.
    Sensitivity(SensitivityError),
    /// The covariance matrix was not square: it had `rows` rows but a row of
    /// width `cols`.
    CovarianceNotSquare { rows: usize, cols: usize },
    /// The covariance dimension does not match the number of parameters.
    CovarianceDimensionMismatch { expected: usize, actual: usize },
    /// A replicate coefficient vector had the wrong width for the parameter set.
    ReplicateDimensionMismatch { expected: usize, actual: usize },
    /// The replicate ensemble supplied to a Monte-Carlo draw was empty.
    EmptyEnsemble,
    /// A supplied covariance, mean, or replicate value was not finite.
    NonFiniteValue,
    /// A supplied band multiplier `z` was not finite.
    NonFiniteMultiplier,
    /// The covariance is not positive semi-definite, so it does not describe a
    /// valid Gaussian and cannot be Cholesky-factored (Monte-Carlo) nor yield a
    /// non-negative delta-method variance.
    NotPositiveSemiDefinite,
    /// A Monte-Carlo forecast was asked for zero samples.
    ZeroSamples,
    /// A confidence level outside the open interval `(0, 1)`.
    InvalidConfidence(f64),
}

impl fmt::Display for PropagateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sensitivity(error) => {
                write!(formatter, "sensitivity integration failed: {error}")
            }
            Self::CovarianceNotSquare { rows, cols } => {
                write!(formatter, "covariance is not square: {rows} rows but a row of width {cols}")
            }
            Self::CovarianceDimensionMismatch { expected, actual } => write!(
                formatter,
                "covariance has dimension {actual} but there are {expected} parameters"
            ),
            Self::ReplicateDimensionMismatch { expected, actual } => write!(
                formatter,
                "a replicate has {actual} coefficients but there are {expected} parameters"
            ),
            Self::EmptyEnsemble => write!(formatter, "the replicate ensemble is empty"),
            Self::NonFiniteValue => {
                write!(formatter, "a covariance, mean, or replicate value is not finite")
            }
            Self::NonFiniteMultiplier => write!(formatter, "the band multiplier z is not finite"),
            Self::NotPositiveSemiDefinite => {
                write!(formatter, "the covariance is not positive semi-definite")
            }
            Self::ZeroSamples => {
                write!(formatter, "a Monte-Carlo forecast needs at least one sample")
            }
            Self::InvalidConfidence(value) => {
                write!(formatter, "confidence {value} is outside the open interval (0, 1)")
            }
        }
    }
}

impl std::error::Error for PropagateError {}

impl From<SensitivityError> for PropagateError {
    fn from(error: SensitivityError) -> Self {
        Self::Sensitivity(error)
    }
}
