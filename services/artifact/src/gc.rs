use crate::{ArtifactError, ArtifactId, cache::ArtifactCache, storage::LocalArtifactStorage};

/// Result of one deterministic retention sweep.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GarbageCollectionReport {
    pub examined: usize,
    pub deleted: Vec<ArtifactId>,
}

pub(crate) fn collect_expired(
    storage: &LocalArtifactStorage,
    cache: &mut ArtifactCache,
    now_unix_seconds: u64,
    dry_run: bool,
) -> Result<GarbageCollectionReport, ArtifactError> {
    let metadata = storage.list_metadata()?;
    let mut report = GarbageCollectionReport { examined: metadata.len(), deleted: Vec::new() };
    for record in metadata {
        if record.is_expired(now_unix_seconds) {
            if !dry_run {
                storage.remove(&record.id)?;
                cache.remove(&record.id);
            }
            report.deleted.push(record.id);
        }
    }
    Ok(report)
}
