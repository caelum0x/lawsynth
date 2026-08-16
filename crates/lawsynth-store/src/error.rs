use std::fmt;

/// Errors produced by the storage layer.
#[derive(Debug)]
pub enum StoreError {
    InvalidKey(String),
    NotFound(String),
    ObjectTooLarge { actual: usize, limit: usize },
    InvalidPart(String),
    Unsupported(String),
    Io(std::io::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(key) => write!(f, "invalid object key: {key}"),
            Self::NotFound(key) => write!(f, "object not found: {key}"),
            Self::ObjectTooLarge { actual, limit } => {
                write!(f, "object size {actual} exceeds configured limit {limit}")
            }
            Self::InvalidPart(reason) => write!(f, "invalid multipart upload: {reason}"),
            Self::Unsupported(feature) => write!(f, "unsupported storage feature: {feature}"),
            Self::Io(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}
impl From<std::io::Error> for StoreError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
