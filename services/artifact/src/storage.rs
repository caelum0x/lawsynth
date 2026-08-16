use crate::{ArtifactError, ArtifactId, ArtifactMetadata, checksum::sha256};
use lawsynth_store::{LocalStore, ObjectKey, ObjectStore, StoreConfig, StoreError};
use std::path::Path;

const DATA_PREFIX: &str = "artifacts/data/";
const METADATA_PREFIX: &str = "artifacts/metadata/";

/// Key-separated, bounded local persistence. Metadata is the publication record;
/// unreferenced data is deliberately invisible and removed by `collect_garbage`.
#[derive(Clone, Debug)]
pub struct LocalArtifactStorage {
    store: LocalStore,
}

impl LocalArtifactStorage {
    pub fn open(root: impl AsRef<Path>, config: StoreConfig) -> Result<Self, ArtifactError> {
        Ok(Self { store: LocalStore::open(root.as_ref().to_path_buf(), config)? })
    }

    pub fn root(&self) -> &Path {
        self.store.root()
    }

    pub fn data_key(id: &ArtifactId) -> ObjectKey {
        ObjectKey::new(format!("{DATA_PREFIX}{id}.bin")).expect("hash address produces a valid key")
    }

    fn metadata_key(id: &ArtifactId) -> ObjectKey {
        ObjectKey::new(format!("{METADATA_PREFIX}{id}.meta"))
            .expect("hash address produces a valid key")
    }

    pub fn put_data_if_absent(
        &self,
        id: &ArtifactId,
        bytes: Vec<u8>,
    ) -> Result<bool, ArtifactError> {
        let key = Self::data_key(id);
        match self.store.get(&key) {
            Ok(existing) => {
                let actual = sha256(&existing.bytes);
                if actual != id.as_str() {
                    return Err(ArtifactError::ChecksumMismatch { id: id.clone(), actual });
                }
                Ok(false)
            }
            Err(StoreError::NotFound(_)) => {
                self.store.put(key, bytes)?;
                Ok(true)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_data(&self, id: &ArtifactId) -> Result<Vec<u8>, ArtifactError> {
        match self.store.get(&Self::data_key(id)) {
            Ok(object) => Ok(object.bytes),
            Err(StoreError::NotFound(_)) => Err(ArtifactError::NotFound(id.clone())),
            Err(error) => Err(error.into()),
        }
    }

    pub fn publish_metadata(&self, metadata: &ArtifactMetadata) -> Result<(), ArtifactError> {
        self.store.put(Self::metadata_key(&metadata.id), metadata.encode().into_bytes())?;
        Ok(())
    }

    pub fn get_metadata(&self, id: &ArtifactId) -> Result<ArtifactMetadata, ArtifactError> {
        let bytes = match self.store.get(&Self::metadata_key(id)) {
            Ok(object) => object.bytes,
            Err(StoreError::NotFound(_)) => return Err(ArtifactError::NotFound(id.clone())),
            Err(error) => return Err(error.into()),
        };
        let metadata = ArtifactMetadata::decode(&bytes)?;
        if &metadata.id != id {
            return Err(ArtifactError::CorruptMetadata(
                "metadata object key does not match contained id".into(),
            ));
        }
        Ok(metadata)
    }

    pub fn list_metadata(&self) -> Result<Vec<ArtifactMetadata>, ArtifactError> {
        self.store
            .list(Some(METADATA_PREFIX))?
            .into_iter()
            .map(|key| {
                let stem = key
                    .as_str()
                    .strip_prefix(METADATA_PREFIX)
                    .and_then(|value| value.strip_suffix(".meta"))
                    .ok_or_else(|| ArtifactError::CorruptMetadata(format!("invalid key {key}")))?;
                self.get_metadata(&ArtifactId::new(stem.to_owned())?)
            })
            .collect()
    }

    pub fn stored_data_bytes(&self) -> Result<u64, ArtifactError> {
        self.store.list(Some(DATA_PREFIX))?.into_iter().try_fold(0_u64, |total, key| {
            let length = self.store.get(&key)?.len() as u64;
            total.checked_add(length).ok_or_else(|| {
                ArtifactError::InvalidConfig("stored byte count overflowed u64".into())
            })
        })
    }

    pub fn remove(&self, id: &ArtifactId) -> Result<bool, ArtifactError> {
        let metadata_removed = self.store.delete(&Self::metadata_key(id))?;
        let data_removed = self.store.delete(&Self::data_key(id))?;
        Ok(metadata_removed || data_removed)
    }
}
