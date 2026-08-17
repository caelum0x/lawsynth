//! The artifact upload path to the object store.
//!
//! Uploads are finalized only after checksum verification: the bytes are hashed
//! before the write, stored, then read back and re-hashed, and the receipt is
//! issued only if the two digests agree (production architecture, section 23:
//! "artifact upload finalizes only after checksum verification"). The digest is
//! the store's content checksum, which detects accidental corruption in transit
//! or at rest -- it is an integrity check, not a security signature.

use lawsynth_store::{ObjectKey, ObjectStore, checksum};

use crate::WorkerError;

/// Proof that an object was stored and verified: its key, content checksum, and
/// byte length.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadReceipt {
    pub key: String,
    pub checksum: u64,
    pub bytes: usize,
}

/// Stores `bytes` at `key`, enforcing a size ceiling and verifying the write by
/// reading it back and comparing checksums. Any mismatch is reported as
/// [`WorkerError::Artifact`] rather than trusted.
pub(crate) fn upload<S: ObjectStore>(
    store: &S,
    key: ObjectKey,
    bytes: Vec<u8>,
    maximum_bytes: usize,
) -> Result<UploadReceipt, WorkerError> {
    if bytes.len() > maximum_bytes {
        return Err(WorkerError::Artifact(format!(
            "artifact of {} bytes exceeds the configured ceiling of {maximum_bytes} bytes",
            bytes.len()
        )));
    }
    let expected = checksum(&bytes);
    let byte_len = bytes.len();
    let key_text = key.as_str().to_owned();

    let stored = store.put(key.clone(), bytes)?;
    if stored.checksum != expected {
        return Err(WorkerError::Artifact(
            "object store reported a checksum that disagrees with the uploaded bytes".into(),
        ));
    }

    // Read-back verification: the upload is only trusted once the store returns
    // the same content we hashed before writing.
    let fetched = store.get(&key)?;
    if !fetched.verify() || fetched.checksum != expected {
        return Err(WorkerError::Artifact(
            "artifact failed read-back checksum verification after upload".into(),
        ));
    }

    Ok(UploadReceipt { key: key_text, checksum: expected, bytes: byte_len })
}
