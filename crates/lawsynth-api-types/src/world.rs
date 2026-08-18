use crate::{ApiValidationError, ProjectId, project::validate_identifier};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorldId(String);

impl WorldId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ApiValidationError> {
        let value = value.into();
        validate_identifier("world_id", &value, 128)?;
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldRevision {
    pub project_id: ProjectId,
    pub world_id: WorldId,
    pub revision: u64,
    pub canonical_sha256: String,
}

impl WorldRevision {
    pub fn new(
        project_id: ProjectId,
        world_id: WorldId,
        revision: u64,
        canonical_sha256: impl Into<String>,
    ) -> Result<Self, ApiValidationError> {
        if revision == 0 {
            return Err(ApiValidationError::Invalid {
                field: "revision",
                reason: "must start at one",
            });
        }
        let canonical_sha256 = canonical_sha256.into();
        if canonical_sha256.len() != 64
            || !canonical_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(ApiValidationError::Invalid {
                field: "canonical_sha256",
                reason: "must be a 64-character hexadecimal SHA-256 digest",
            });
        }
        Ok(Self {
            project_id,
            world_id,
            revision,
            canonical_sha256: canonical_sha256.to_ascii_lowercase(),
        })
    }
}
