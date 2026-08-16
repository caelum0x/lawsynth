use crate::ArtifactError;

/// Immutable resource limits for a local artifact installation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactLimits {
    pub max_total_bytes: u64,
    pub max_multipart_parts: u32,
    pub max_multipart_bytes: usize,
}

impl Default for ArtifactLimits {
    fn default() -> Self {
        Self {
            max_total_bytes: 4 * 1024 * 1024 * 1024,
            max_multipart_parts: 10_000,
            max_multipart_bytes: 128 * 1024 * 1024,
        }
    }
}

impl ArtifactLimits {
    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.max_total_bytes == 0
            || self.max_multipart_parts == 0
            || self.max_multipart_bytes == 0
        {
            return Err(ArtifactError::InvalidConfig(
                "storage and multipart limits must be positive".into(),
            ));
        }
        Ok(())
    }
}
