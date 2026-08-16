use crate::ArtifactId;
use lawsynth_store::StoreError;
use std::fmt;

/// Failures that callers can act on without inspecting storage internals.
#[derive(Debug)]
pub enum ArtifactError {
    InvalidConfig(String),
    InvalidArtifactId(String),
    InvalidMetadata(String),
    InvalidUpload(String),
    NotFound(ArtifactId),
    Expired(ArtifactId),
    ChecksumMismatch { id: ArtifactId, actual: String },
    CapacityExceeded { requested: u64, available: u64 },
    CorruptMetadata(String),
    Store(StoreError),
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => write!(f, "invalid artifact configuration: {reason}"),
            Self::InvalidArtifactId(id) => write!(f, "invalid artifact id: {id}"),
            Self::InvalidMetadata(reason) => write!(f, "invalid artifact metadata: {reason}"),
            Self::InvalidUpload(reason) => write!(f, "invalid multipart upload: {reason}"),
            Self::NotFound(id) => write!(f, "artifact not found: {id}"),
            Self::Expired(id) => write!(f, "artifact has expired: {id}"),
            Self::ChecksumMismatch { id, actual } => {
                write!(f, "artifact checksum mismatch for {id}; observed {actual}")
            }
            Self::CapacityExceeded { requested, available } => write!(
                f,
                "artifact storage capacity exceeded: requested {requested} bytes, {available} bytes available"
            ),
            Self::CorruptMetadata(reason) => write!(f, "corrupt artifact metadata: {reason}"),
            Self::Store(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ArtifactError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}

impl From<StoreError> for ArtifactError {
    fn from(value: StoreError) -> Self {
        Self::Store(value)
    }
}
