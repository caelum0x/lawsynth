use crate::{ArtifactError, ArtifactId, ArtifactMetadata, storage::LocalArtifactStorage};

/// Metadata catalog backed by durable local publication records rather than transient memory.
#[derive(Clone, Debug)]
pub struct ArtifactCatalog {
    storage: LocalArtifactStorage,
}

impl ArtifactCatalog {
    pub(crate) fn new(storage: LocalArtifactStorage) -> Self {
        Self { storage }
    }

    pub fn get(&self, id: &ArtifactId) -> Result<ArtifactMetadata, ArtifactError> {
        self.storage.get_metadata(id)
    }

    pub fn list(&self) -> Result<Vec<ArtifactMetadata>, ArtifactError> {
        self.storage.list_metadata()
    }

    pub fn count(&self) -> Result<usize, ArtifactError> {
        Ok(self.list()?.len())
    }
}
