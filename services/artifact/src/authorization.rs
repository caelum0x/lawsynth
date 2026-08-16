use crate::ArtifactError;

/// Operations understood by the local core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessAction {
    Read,
    Write,
    Delete,
    CollectGarbage,
}

/// Deliberate security boundary: this crate trusts only the in-process local caller.
/// Authentication and remote principal propagation belong to a separately implemented
/// network adapter, rather than being implied by a boolean "authorized" flag.
#[derive(Clone, Debug, Default)]
pub struct LocalOnlyAuthorizer;

impl LocalOnlyAuthorizer {
    pub fn authorize(&self, caller: &str, _action: AccessAction) -> Result<(), ArtifactError> {
        if caller == "local" {
            Ok(())
        } else {
            Err(ArtifactError::InvalidMetadata(
                "the local artifact core accepts only the 'local' principal".into(),
            ))
        }
    }
}
