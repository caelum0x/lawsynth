use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptimizationError {
    EmptyInput,
    LengthMismatch,
    NonFiniteInput,
    DegeneratePredictor,
    InvalidConfig,
    InvalidBounds,
    NonFiniteObjective,
}

impl fmt::Display for OptimizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(formatter, "optimizer input cannot be empty"),
            Self::LengthMismatch => {
                write!(formatter, "predictions and targets must have equal lengths")
            }
            Self::NonFiniteInput => write!(formatter, "optimizer input must be finite"),
            Self::DegeneratePredictor => write!(formatter, "candidate prediction has no variation"),
            Self::InvalidConfig => write!(formatter, "optimizer configuration is invalid"),
            Self::InvalidBounds => write!(formatter, "parameter bounds are invalid"),
            Self::NonFiniteObjective => write!(formatter, "objective returned a non-finite value"),
        }
    }
}

impl std::error::Error for OptimizationError {}
