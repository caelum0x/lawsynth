use std::{fmt, str::FromStr};

/// Semantic engine version embedded in reproducible artifacts and run metadata.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct EngineVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

/// The version of the executable World semantics implemented by this build.
pub const CURRENT_ENGINE_VERSION: EngineVersion = EngineVersion::new(0, 1, 0);

impl EngineVersion {
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }

    /// Returns whether artifacts share a compatible major-version contract.
    pub const fn is_compatible_with(self, other: Self) -> bool {
        self.major == other.major
    }
}

impl fmt::Display for EngineVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionParseError;

impl fmt::Display for VersionParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "version must use MAJOR.MINOR.PATCH decimal notation")
    }
}

impl std::error::Error for VersionParseError {}

impl FromStr for EngineVersion {
    type Err = VersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split('.');
        let major = parts.next().and_then(|part| part.parse().ok());
        let minor = parts.next().and_then(|part| part.parse().ok());
        let patch = parts.next().and_then(|part| part.parse().ok());
        if parts.next().is_some() {
            return Err(VersionParseError);
        }
        match (major, minor, patch) {
            (Some(major), Some(minor), Some(patch)) => Ok(Self::new(major, minor, patch)),
            _ => Err(VersionParseError),
        }
    }
}
