use std::{collections::BTreeMap, fmt};

/// Deterministic execution stages shared by discovery-oriented workflows.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProgressStage {
    Input,
    Profiling,
    Preprocessing,
    Differentiating,
    Features,
    Fitting,
    Scoring,
    Finalizing,
}

/// A monotonic update for one stage. Progress is intentionally only monotonic
/// within a stage; alternative branches may report stages in a different order.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressEvent {
    pub sequence: u64,
    pub stage: ProgressStage,
    pub fraction: f64,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgressError {
    InvalidFraction,
    NonMonotonic,
}

impl fmt::Display for ProgressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFraction => {
                write!(formatter, "progress fraction must be finite and between zero and one")
            }
            Self::NonMonotonic => write!(formatter, "progress cannot decrease within a stage"),
        }
    }
}

impl std::error::Error for ProgressError {}

/// State holder that assigns deterministic event sequence numbers and enforces
/// the per-stage monotonicity contract.
#[derive(Clone, Debug, Default)]
pub struct ProgressTracker {
    next_sequence: u64,
    completed: BTreeMap<ProgressStage, f64>,
}

impl ProgressTracker {
    pub fn report(
        &mut self,
        stage: ProgressStage,
        fraction: f64,
        message: impl Into<String>,
    ) -> Result<ProgressEvent, ProgressError> {
        if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
            return Err(ProgressError::InvalidFraction);
        }
        if self.completed.get(&stage).is_some_and(|previous| fraction < *previous) {
            return Err(ProgressError::NonMonotonic);
        }
        self.completed.insert(stage, fraction);
        let event = ProgressEvent {
            sequence: self.next_sequence,
            stage,
            fraction,
            message: message.into(),
        };
        self.next_sequence += 1;
        Ok(event)
    }

    pub fn fraction(&self, stage: ProgressStage) -> Option<f64> {
        self.completed.get(&stage).copied()
    }
}
