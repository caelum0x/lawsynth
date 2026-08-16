use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreprocessError {
    ZeroRadius,
    ResampleOutOfBounds,
    ConstantColumn(String),
    MissingScaleColumn(String),
    InvalidTargetTime,
    ImputationLengthMismatch,
    NoObservedValues,
    MissingBoundaryValue,
    NonFiniteImputationValue,
    AlignmentLengthMismatch,
    InvalidAlignmentSource,
}

impl fmt::Display for PreprocessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroRadius => write!(formatter, "smoothing radius must be positive"),
            Self::ResampleOutOfBounds => {
                write!(
                    formatter,
                    "resampling target is outside the source time range"
                )
            }
            Self::ConstantColumn(column) => {
                write!(formatter, "cannot standardize constant column '{column}'")
            }
            Self::MissingScaleColumn(column) => {
                write!(
                    formatter,
                    "scale report has no constants for column '{column}'"
                )
            }
            Self::InvalidTargetTime => {
                write!(formatter, "pipeline resampling time axis is invalid")
            }
            Self::ImputationLengthMismatch => write!(
                formatter,
                "imputation time and values must have equal lengths"
            ),
            Self::NoObservedValues => {
                write!(formatter, "imputation requires at least one observed value")
            }
            Self::MissingBoundaryValue => write!(
                formatter,
                "linear interpolation cannot impute an unbounded edge gap"
            ),
            Self::NonFiniteImputationValue => write!(
                formatter,
                "imputation observations and time values must be finite"
            ),
            Self::AlignmentLengthMismatch => {
                write!(
                    formatter,
                    "alignment source time and values must have equal lengths"
                )
            }
            Self::InvalidAlignmentSource => write!(
                formatter,
                "alignment source time must be finite and strictly increasing"
            ),
        }
    }
}

impl std::error::Error for PreprocessError {}
