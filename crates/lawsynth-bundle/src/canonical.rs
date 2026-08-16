use crate::BundleError;
use std::collections::BTreeSet;

/// Validates portable entry paths and returns their canonical lexical order.
pub fn canonical_entry_order(
    paths: impl IntoIterator<Item = String>,
) -> Result<Vec<String>, BundleError> {
    let mut ordered = BTreeSet::new();
    for path in paths {
        if path.is_empty()
            || path.starts_with('/')
            || path.contains('\\')
            || path
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | ".."))
        {
            return Err(BundleError::InvalidPath(path));
        }
        if !ordered.insert(path.clone()) {
            return Err(BundleError::InvalidPath(path));
        }
    }
    Ok(ordered.into_iter().collect())
}
