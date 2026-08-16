use std::{fmt, io};

use lawsynth_world::WorldError;

#[derive(Debug)]
pub enum BundleError {
    Io(io::Error),
    InvalidArchive(&'static str),
    InvalidPath(String),
    ChecksumMismatch(String),
    MissingEntry(&'static str),
    InvalidWorld(&'static str),
    World(WorldError),
}

impl fmt::Display for BundleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::InvalidArchive(reason) => write!(formatter, "invalid .lsworld archive: {reason}"),
            Self::InvalidPath(path) => write!(formatter, "invalid archive path '{path}'"),
            Self::ChecksumMismatch(path) => write!(formatter, "checksum mismatch for '{path}'"),
            Self::MissingEntry(path) => {
                write!(formatter, "required bundle entry '{path}' is missing")
            }
            Self::InvalidWorld(reason) => write!(formatter, "invalid world encoding: {reason}"),
            Self::World(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for BundleError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::World(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for BundleError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<WorldError> for BundleError {
    fn from(error: WorldError) -> Self {
        Self::World(error)
    }
}
