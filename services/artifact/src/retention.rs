use crate::ArtifactError;

/// Retention controls expressed as a Unix timestamp. `None` means no expiry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Retention {
    pub expires_at_unix_seconds: Option<u64>,
}

impl Retention {
    pub fn until(expires_at_unix_seconds: u64) -> Self {
        Self { expires_at_unix_seconds: Some(expires_at_unix_seconds) }
    }

    pub fn validate(&self) -> Result<(), ArtifactError> {
        Ok(())
    }

    pub fn is_expired(&self, now_unix_seconds: u64) -> bool {
        self.expires_at_unix_seconds.is_some_and(|expires_at| expires_at <= now_unix_seconds)
    }
}
