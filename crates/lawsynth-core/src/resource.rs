use std::fmt;

/// Explicit work limits for bounded in-process engine operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub max_samples: usize,
    pub max_columns: usize,
    pub max_features: usize,
    pub max_candidates: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_samples: 1_000_000,
            max_columns: 1_024,
            max_features: 50_000,
            max_candidates: 10_000,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResourceLimitError {
    InvalidLimit,
    Exceeded { resource: &'static str, actual: usize, limit: usize },
}

impl fmt::Display for ResourceLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimit => write!(formatter, "resource limits must be nonzero"),
            Self::Exceeded { resource, actual, limit } => {
                write!(formatter, "{resource} limit exceeded: {actual} > {limit}")
            }
        }
    }
}

impl std::error::Error for ResourceLimitError {}

impl ResourceLimits {
    pub fn validate(self) -> Result<Self, ResourceLimitError> {
        if self.max_samples == 0
            || self.max_columns == 0
            || self.max_features == 0
            || self.max_candidates == 0
        {
            return Err(ResourceLimitError::InvalidLimit);
        }
        Ok(self)
    }

    pub fn validate_dataset(
        self,
        samples: usize,
        columns: usize,
    ) -> Result<(), ResourceLimitError> {
        self.validate()?;
        self.check("samples", samples, self.max_samples)?;
        self.check("columns", columns, self.max_columns)
    }

    pub fn validate_feature_count(self, features: usize) -> Result<(), ResourceLimitError> {
        self.validate()?;
        self.check("features", features, self.max_features)
    }

    pub fn validate_candidate_count(self, candidates: usize) -> Result<(), ResourceLimitError> {
        self.validate()?;
        self.check("candidates", candidates, self.max_candidates)
    }

    fn check(
        self,
        resource: &'static str,
        actual: usize,
        limit: usize,
    ) -> Result<(), ResourceLimitError> {
        (actual <= limit).then_some(()).ok_or(ResourceLimitError::Exceeded {
            resource,
            actual,
            limit,
        })
    }
}
