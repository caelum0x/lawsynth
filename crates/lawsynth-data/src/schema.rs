use lawsynth_core::Identifier;

/// Deterministic numeric column schema for a time-series dataset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatasetSchema {
    pub columns: Vec<Identifier>,
}
