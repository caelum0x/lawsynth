use std::fmt;

#[derive(Debug)]
pub enum DiscoveryError {
    NoStates,
    TooFewSamples,
    Checkpoint(String),
    Cancelled,
    MissingState(String),
    Profile(String),
    Differentiate(String),
    Preprocess(String),
    Features(String),
    Sparse(String),
    Symbolic(String),
    World(String),
    Graph(String),
    Score(String),
    Regime(String),
    Resource(String),
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoStates => write!(formatter, "discovery requires at least one state"),
            Self::TooFewSamples => write!(formatter, "discovery requires at least three samples"),
            Self::Checkpoint(error) => write!(formatter, "checkpoint error: {error}"),
            Self::Cancelled => write!(formatter, "discovery was cancelled"),
            Self::MissingState(id) => write!(formatter, "state '{id}' is absent from the dataset"),
            Self::Profile(error)
            | Self::Differentiate(error)
            | Self::Preprocess(error)
            | Self::Features(error)
            | Self::Sparse(error)
            | Self::Symbolic(error)
            | Self::World(error)
            | Self::Graph(error)
            | Self::Score(error) => error.fmt(formatter),
            Self::Regime(error) => write!(formatter, "regime segmentation error: {error}"),
            Self::Resource(error) => write!(formatter, "resource limit: {error}"),
        }
    }
}

impl std::error::Error for DiscoveryError {}
