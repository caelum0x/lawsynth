use crate::{ArtifactError, limits::ArtifactLimits};
use lawsynth_store::StoreConfig;
use std::path::PathBuf;

/// Explicit local backend configuration. No network endpoint is accepted here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactConfig {
    pub root: PathBuf,
    pub store: StoreConfig,
    pub limits: ArtifactLimits,
    pub cache_capacity_bytes: usize,
}

impl ArtifactConfig {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let store = StoreConfig::default();
        Self {
            root: root.into(),
            cache_capacity_bytes: store.cache_capacity_bytes,
            store,
            limits: ArtifactLimits::default(),
        }
    }

    pub fn validate(&self) -> Result<(), ArtifactError> {
        if self.root.as_os_str().is_empty() {
            return Err(ArtifactError::InvalidConfig("root must not be empty".into()));
        }
        self.store.validate()?;
        self.limits.validate()?;
        if self.store.max_object_bytes < 512 {
            return Err(ArtifactError::InvalidConfig(
                "max object bytes must leave 512 bytes for durable metadata records".into(),
            ));
        }
        if self.store.max_object_bytes > self.limits.max_multipart_bytes {
            return Err(ArtifactError::InvalidConfig(
                "max multipart bytes must be at least max object bytes".into(),
            ));
        }
        Ok(())
    }
}
