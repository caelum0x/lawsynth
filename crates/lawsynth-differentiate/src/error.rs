use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DifferentiationError {
    TooFewSamples,
    LengthMismatch,
    InvalidWindow,
    InvalidTotalVariationConfig,
    IrregularTimeAxis,
    SingularFit,
}

impl fmt::Display for DifferentiationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewSamples => write!(formatter, "at least two samples are required"),
            Self::LengthMismatch => write!(formatter, "time and values must have equal lengths"),
            Self::InvalidWindow => write!(
                formatter,
                "Savitzky-Golay window must be odd and at least three"
            ),
            Self::InvalidTotalVariationConfig => write!(
                formatter,
                "total-variation lambda must be finite and positive and iterations must be nonzero"
            ),
            Self::IrregularTimeAxis => {
                write!(
                    formatter,
                    "spectral differentiation requires a regular time axis"
                )
            }
            Self::SingularFit => {
                write!(formatter, "Savitzky-Golay local polynomial fit is singular")
            }
        }
    }
}

impl std::error::Error for DifferentiationError {}
