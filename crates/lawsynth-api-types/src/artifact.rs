use crate::{ApiValidationError, ProjectId, RunId, project::validate_identifier};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactId(String);

impl ArtifactId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ApiValidationError> {
        let value = value.into();
        validate_identifier("artifact_id", &value, 128)?;
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactMediaType {
    Json,
    Csv,
    Parquet,
    Zip,
    Text,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactDescriptor {
    pub id: ArtifactId,
    pub project_id: ProjectId,
    pub run_id: Option<RunId>,
    pub media_type: ArtifactMediaType,
    pub byte_len: u64,
    pub sha256: String,
}

impl ArtifactDescriptor {
    pub fn new(
        id: ArtifactId,
        project_id: ProjectId,
        run_id: Option<RunId>,
        media_type: ArtifactMediaType,
        byte_len: u64,
        sha256: impl Into<String>,
    ) -> Result<Self, ApiValidationError> {
        let sha256 = sha256.into();
        if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ApiValidationError::Invalid {
                field: "sha256",
                reason: "must be a 64-character hexadecimal SHA-256 digest",
            });
        }
        Ok(Self {
            id,
            project_id,
            run_id,
            media_type,
            byte_len,
            sha256: sha256.to_ascii_lowercase(),
        })
    }
}
