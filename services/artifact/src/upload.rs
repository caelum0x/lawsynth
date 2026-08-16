use crate::{
    Artifact, ArtifactCatalog, ArtifactConfig, ArtifactError, ArtifactId, ArtifactMetadata,
    GarbageCollectionReport, HealthReport, Telemetry, UploadId, UploadOptions, cache,
    checksum::sha256, download, gc, health, multipart::PendingUpload,
    storage::LocalArtifactStorage,
};
use lawsynth_store::StoreError;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct MutableState {
    next_upload_sequence: u64,
    uploads: BTreeMap<UploadId, PendingUpload>,
    cache: cache::ArtifactCache,
}

/// Cohesive local artifact lifecycle service backed by atomic local files.
#[derive(Clone, Debug)]
pub struct ArtifactService {
    config: ArtifactConfig,
    storage: LocalArtifactStorage,
    catalog: ArtifactCatalog,
    state: Arc<Mutex<MutableState>>,
    telemetry: Telemetry,
}

impl ArtifactService {
    pub fn open(config: ArtifactConfig) -> Result<Self, ArtifactError> {
        config.validate()?;
        let storage = LocalArtifactStorage::open(&config.root, config.store.clone())?;
        let catalog = ArtifactCatalog::new(storage.clone());
        Ok(Self {
            state: Arc::new(Mutex::new(MutableState {
                next_upload_sequence: 1,
                uploads: BTreeMap::new(),
                cache: cache::ArtifactCache::new(config.cache_capacity_bytes),
            })),
            config,
            storage,
            catalog,
            telemetry: Telemetry::default(),
        })
    }

    pub fn config(&self) -> &ArtifactConfig {
        &self.config
    }
    pub fn root(&self) -> &std::path::Path {
        self.storage.root()
    }
    pub fn catalog(&self) -> &ArtifactCatalog {
        &self.catalog
    }
    pub fn telemetry(&self) -> &Telemetry {
        &self.telemetry
    }
    pub(crate) fn storage(&self) -> &LocalArtifactStorage {
        &self.storage
    }

    /// Commits a verified immutable object and then atomically publishes its metadata.
    pub fn ingest(
        &self,
        bytes: Vec<u8>,
        options: UploadOptions,
        now_unix_seconds: u64,
    ) -> Result<ArtifactMetadata, ArtifactError> {
        let _state = self.state.lock().expect("artifact state lock poisoned");
        self.ingest_locked(bytes, options, now_unix_seconds)
    }

    fn ingest_locked(
        &self,
        bytes: Vec<u8>,
        options: UploadOptions,
        now_unix_seconds: u64,
    ) -> Result<ArtifactMetadata, ArtifactError> {
        options.validate()?;
        if bytes.len() > self.config.store.max_object_bytes {
            return Err(StoreError::ObjectTooLarge {
                actual: bytes.len(),
                limit: self.config.store.max_object_bytes,
            }
            .into());
        }
        let id = ArtifactId::new(sha256(&bytes))?;
        match self.storage.get_metadata(&id) {
            Ok(existing) => {
                let present = self.storage.get_data(&id)?;
                let actual = sha256(&present);
                if actual != id.as_str() {
                    self.telemetry.checksum_failure();
                    return Err(ArtifactError::ChecksumMismatch { id, actual });
                }
                if present.len() as u64 != existing.size_bytes {
                    return Err(ArtifactError::CorruptMetadata(format!(
                        "artifact {id} records {} bytes but object contains {}",
                        existing.size_bytes,
                        present.len()
                    )));
                }
                return Ok(existing);
            }
            Err(ArtifactError::NotFound(_)) => {}
            Err(error) => return Err(error),
        }
        let data_exists = match self.storage.get_data(&id) {
            Ok(_) => true,
            Err(ArtifactError::NotFound(_)) => false,
            Err(error) => return Err(error),
        };
        if !data_exists {
            let used = self.storage.stored_data_bytes()?;
            let requested = bytes.len() as u64;
            let available = self.config.limits.max_total_bytes.saturating_sub(used);
            if requested > available {
                return Err(ArtifactError::CapacityExceeded { requested, available });
            }
        }
        self.storage.put_data_if_absent(&id, bytes.clone())?;
        let metadata = ArtifactMetadata::new(id, bytes.len() as u64, now_unix_seconds, options)?;
        self.storage.publish_metadata(&metadata)?;
        self.telemetry.upload();
        Ok(metadata)
    }

    pub fn get(&self, id: &ArtifactId, now_unix_seconds: u64) -> Result<Artifact, ArtifactError> {
        let metadata = self.storage.get_metadata(id)?;
        if metadata.is_expired(now_unix_seconds) {
            return Err(ArtifactError::Expired(id.clone()));
        }
        let mut state = self.state.lock().expect("artifact state lock poisoned");
        if let Some(cached) = state.cache.get(id) {
            if download::metadata_is_current(&cached, &metadata) {
                self.telemetry.download();
                return Ok(cached);
            }
        }
        let artifact = download::read_verified(&self.storage, id, now_unix_seconds)?;
        state.cache.insert(artifact.clone());
        self.telemetry.download();
        Ok(artifact)
    }

    pub fn delete(&self, id: &ArtifactId) -> Result<bool, ArtifactError> {
        let mut state = self.state.lock().expect("artifact state lock poisoned");
        let removed = self.storage.remove(id)?;
        state.cache.remove(id);
        Ok(removed)
    }

    pub fn begin_multipart(&self, options: UploadOptions) -> Result<UploadId, ArtifactError> {
        let mut state = self.state.lock().expect("artifact state lock poisoned");
        let id = UploadId::new(state.next_upload_sequence);
        state.next_upload_sequence = state.next_upload_sequence.wrapping_add(1).max(1);
        state.uploads.insert(id.clone(), PendingUpload::new(options)?);
        Ok(id)
    }

    pub fn add_multipart_part(
        &self,
        id: &UploadId,
        number: u32,
        bytes: Vec<u8>,
    ) -> Result<(), ArtifactError> {
        let mut state = self.state.lock().expect("artifact state lock poisoned");
        let upload = state
            .uploads
            .get_mut(id)
            .ok_or_else(|| ArtifactError::InvalidUpload(format!("unknown upload {id}")))?;
        upload.add_part(
            number,
            bytes,
            self.config.limits.max_multipart_bytes,
            self.config.limits.max_multipart_parts,
        )
    }

    /// A failed completion retains the session; a successful completion removes it only
    /// after bytes and metadata have been published.
    pub fn complete_multipart(
        &self,
        id: &UploadId,
        now_unix_seconds: u64,
    ) -> Result<ArtifactMetadata, ArtifactError> {
        let mut state = self.state.lock().expect("artifact state lock poisoned");
        let upload = state
            .uploads
            .get(id)
            .ok_or_else(|| ArtifactError::InvalidUpload(format!("unknown upload {id}")))?;
        let bytes = upload.assemble(self.config.store.max_object_bytes)?;
        let metadata = self.ingest_locked(bytes, upload.options.clone(), now_unix_seconds)?;
        state.uploads.remove(id);
        Ok(metadata)
    }

    pub fn abort_multipart(&self, id: &UploadId) -> bool {
        self.state.lock().expect("artifact state lock poisoned").uploads.remove(id).is_some()
    }

    pub fn collect_garbage(
        &self,
        now_unix_seconds: u64,
        dry_run: bool,
    ) -> Result<GarbageCollectionReport, ArtifactError> {
        let mut state = self.state.lock().expect("artifact state lock poisoned");
        let report =
            gc::collect_expired(&self.storage, &mut state.cache, now_unix_seconds, dry_run)?;
        if !dry_run {
            self.telemetry.gc_deleted(report.deleted.len() as u64);
        }
        Ok(report)
    }

    pub fn health(&self) -> Result<HealthReport, ArtifactError> {
        health::check(self)
    }
}
