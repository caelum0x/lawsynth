use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_expr::EvaluationError;

#[derive(Clone, Debug, PartialEq)]
pub enum SimulationError {
    InvalidTimeGrid,
    MissingInitialState(Identifier),
    UnknownInitialState(Identifier),
    UnknownParameterOverride(Identifier),
    UnknownInput(Identifier),
    InputTargetsState(Identifier),
    InvalidInterventionTime { name: Identifier, time: f64 },
    NonFiniteInput { name: Identifier, value: f64 },
    TimeResolutionLoss,
    Evaluation(EvaluationError),
}

impl fmt::Display for SimulationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeGrid => write!(
                formatter,
                "simulation end must be after start and step must be finite and positive"
            ),
            Self::MissingInitialState(id) => {
                write!(formatter, "missing initial value for state '{id}'")
            }
            Self::UnknownInitialState(id) => {
                write!(formatter, "initial value supplied for non-state variable '{id}'")
            }
            Self::UnknownParameterOverride(id) => {
                write!(formatter, "parameter override supplied for unknown parameter '{id}'")
            }
            Self::UnknownInput(id) => {
                write!(formatter, "input supplied for unknown variable '{id}'")
            }
            Self::InputTargetsState(id) => {
                write!(formatter, "input '{id}' conflicts with a simulated state")
            }
            Self::InvalidInterventionTime { name, time } => {
                write!(formatter, "intervention for '{name}' has invalid time {time}")
            }
            Self::NonFiniteInput { name, value } => {
                write!(formatter, "input '{name}' must be finite, got {value}")
            }
            Self::TimeResolutionLoss => write!(
                formatter,
                "time step is too small to advance the current floating-point timestamp"
            ),
            Self::Evaluation(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SimulationError {}

impl From<EvaluationError> for SimulationError {
    fn from(error: EvaluationError) -> Self {
        Self::Evaluation(error)
    }
}
