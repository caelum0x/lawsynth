use crate::{ObjectKey, ObjectStore, StoreError};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GcReport {
    pub examined: usize,
    pub deleted: Vec<ObjectKey>,
}
/// Delete objects not reachable from an externally supplied retained-key set.
pub fn collect_unreferenced<S: ObjectStore>(
    store: &S,
    retained: &BTreeSet<ObjectKey>,
    prefix: Option<&str>,
    dry_run: bool,
) -> Result<GcReport, StoreError> {
    let keys = store.list(prefix)?;
    let mut report = GcReport { examined: keys.len(), deleted: Vec::new() };
    for key in keys {
        if !retained.contains(&key) {
            if !dry_run {
                store.delete(&key)?;
            }
            report.deleted.push(key);
        }
    }
    Ok(report)
}
