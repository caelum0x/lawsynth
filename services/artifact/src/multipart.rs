use crate::{ArtifactError, UploadOptions};
use std::collections::BTreeMap;

/// Opaque local process session identifier. Sessions are intentionally not durable:
/// callers must resume at a higher workflow layer after a process restart.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct UploadId(String);

impl UploadId {
    pub(crate) fn new(sequence: u64) -> Self {
        Self(format!("upload-{sequence:016x}"))
    }

    /// Reconstructs a session identifier from an opaque token supplied by a caller
    /// (for example, a transport path segment). The canonical form is
    /// `upload-<16 lowercase hex digits>`; anything else is rejected so that
    /// malformed tokens never reach the in-process session map.
    pub fn parse(token: &str) -> Result<Self, ArtifactError> {
        let digits = token
            .strip_prefix("upload-")
            .filter(|digits| {
                digits.len() == 16
                    && digits
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            })
            .ok_or_else(|| ArtifactError::InvalidUpload(format!("malformed upload id {token}")))?;
        // A valid 16-digit lowercase hex string always parses; re-emit the canonical form.
        let sequence = u64::from_str_radix(digits, 16)
            .map_err(|_| ArtifactError::InvalidUpload(format!("malformed upload id {token}")))?;
        Ok(Self::new(sequence))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UploadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PendingUpload {
    pub options: UploadOptions,
    pub parts: BTreeMap<u32, Vec<u8>>,
}

impl PendingUpload {
    pub fn new(options: UploadOptions) -> Result<Self, ArtifactError> {
        options.validate()?;
        Ok(Self { options, parts: BTreeMap::new() })
    }

    pub fn add_part(
        &mut self,
        number: u32,
        bytes: Vec<u8>,
        max_part_bytes: usize,
        max_parts: u32,
    ) -> Result<(), ArtifactError> {
        if number == 0 || number > max_parts {
            return Err(ArtifactError::InvalidUpload(format!(
                "part number must be in 1..={max_parts}"
            )));
        }
        if bytes.len() > max_part_bytes {
            return Err(ArtifactError::InvalidUpload(format!(
                "part {number} exceeds the configured part limit"
            )));
        }
        if self.parts.insert(number, bytes).is_some() {
            return Err(ArtifactError::InvalidUpload(format!(
                "part {number} was already uploaded"
            )));
        }
        Ok(())
    }

    pub fn assemble(&self, max_total_bytes: usize) -> Result<Vec<u8>, ArtifactError> {
        if self.parts.is_empty() {
            return Err(ArtifactError::InvalidUpload(
                "cannot complete an upload without parts".into(),
            ));
        }
        let mut expected = 1;
        let mut result = Vec::new();
        for (number, bytes) in &self.parts {
            if *number != expected {
                return Err(ArtifactError::InvalidUpload(format!("missing part {expected}")));
            }
            expected += 1;
            if result.len().checked_add(bytes.len()).is_none_or(|size| size > max_total_bytes) {
                return Err(ArtifactError::InvalidUpload(
                    "multipart payload exceeds object limit".into(),
                ));
            }
            result.extend_from_slice(bytes);
        }
        Ok(result)
    }
}
