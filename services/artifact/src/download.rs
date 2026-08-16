use crate::{
    Artifact, ArtifactError, ArtifactId, ArtifactMetadata, checksum::sha256,
    storage::LocalArtifactStorage,
};

/// Reads immutable bytes only after their durable publication record is verified.
pub(crate) fn read_verified(
    storage: &LocalArtifactStorage,
    id: &ArtifactId,
    now_unix_seconds: u64,
) -> Result<Artifact, ArtifactError> {
    let metadata = storage.get_metadata(id)?;
    if metadata.is_expired(now_unix_seconds) {
        return Err(ArtifactError::Expired(id.clone()));
    }
    let bytes = storage.get_data(id)?;
    let actual = sha256(&bytes);
    if actual != metadata.sha256 || actual != id.as_str() {
        return Err(ArtifactError::ChecksumMismatch { id: id.clone(), actual });
    }
    if bytes.len() as u64 != metadata.size_bytes {
        return Err(ArtifactError::CorruptMetadata(format!(
            "artifact {id} records {} bytes but object contains {}",
            metadata.size_bytes,
            bytes.len()
        )));
    }
    Ok(Artifact { metadata, bytes })
}

pub(crate) fn metadata_is_current(cached: &Artifact, metadata: &ArtifactMetadata) -> bool {
    cached.metadata == *metadata
}
