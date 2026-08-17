//! Post-job scratch cleanup.
//!
//! A running job may stage intermediate objects under a per-job scratch prefix
//! in the object store. Once the job reaches a terminal state that scratch is
//! dead weight, so cleanup enumerates and deletes it. Lifecycle checkpoints and
//! recorded artifact manifests live under different prefixes and are never
//! touched: cleanup only reclaims scratch. It is idempotent -- deleting an
//! already-empty scratch namespace reports zero deletions and is not an error.

use lawsynth_store::{ObjectKey, ObjectStore};

use crate::WorkerError;

/// The object-store prefix under which per-job scratch is staged.
const PREFIX: &str = "worker/scratch/";

/// The outcome of a cleanup pass for one job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanupReport {
    pub job_id: String,
    pub deleted: usize,
}

/// Builds the scratch key for a named object belonging to `job_id`, so callers
/// stage and reclaim scratch under one agreed convention.
pub fn scratch_key(job_id: &str, name: &str) -> Result<ObjectKey, WorkerError> {
    ObjectKey::new(format!("{PREFIX}{job_id}/{name}")).map_err(WorkerError::from)
}

/// Deletes every scratch object belonging to `job_id`, returning how many were
/// removed. Safe to call more than once.
pub(crate) fn cleanup<S: ObjectStore>(
    store: &S,
    job_id: &str,
) -> Result<CleanupReport, WorkerError> {
    let prefix = format!("{PREFIX}{job_id}/");
    let keys = store.list(Some(&prefix))?;
    let mut deleted = 0;
    for key in keys {
        if store.delete(&key)? {
            deleted += 1;
        }
    }
    Ok(CleanupReport { job_id: job_id.to_owned(), deleted })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_store::MemoryStore;

    #[test]
    fn deletes_only_the_targeted_jobs_scratch_and_is_idempotent() {
        let store = MemoryStore::default();
        store.put(scratch_key("job-a", "step-1").unwrap(), b"a1".to_vec()).unwrap();
        store.put(scratch_key("job-a", "step-2").unwrap(), b"a2".to_vec()).unwrap();
        store.put(scratch_key("job-b", "step-1").unwrap(), b"b1".to_vec()).unwrap();

        let report = cleanup(&store, "job-a").unwrap();
        assert_eq!(report.deleted, 2);
        // job-b's scratch is untouched.
        assert!(store.contains(&scratch_key("job-b", "step-1").unwrap()).unwrap());

        // Second pass finds nothing and is not an error.
        assert_eq!(cleanup(&store, "job-a").unwrap().deleted, 0);
    }

    #[test]
    fn cleanup_of_unknown_job_reports_zero() {
        let store = MemoryStore::default();
        assert_eq!(cleanup(&store, "ghost").unwrap().deleted, 0);
    }
}
