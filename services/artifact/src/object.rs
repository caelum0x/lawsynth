use crate::{ArtifactError, is_sha256_hex};

/// Content address of an artifact. It is always a canonical SHA-256 digest.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct ArtifactId(String);

impl ArtifactId {
    pub fn new(value: impl Into<String>) -> Result<Self, ArtifactError> {
        let value = value.into();
        if !is_sha256_hex(&value) {
            return Err(ArtifactError::InvalidArtifactId(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Caller-provided, validated properties retained alongside the immutable bytes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UploadOptions {
    pub content_type: Option<String>,
    pub retention: crate::Retention,
}

impl UploadOptions {
    pub fn validate(&self) -> Result<(), ArtifactError> {
        if let Some(content_type) = &self.content_type {
            if content_type.is_empty()
                || content_type.len() > 255
                || !content_type.bytes().all(|byte| {
                    byte.is_ascii_graphic() && byte != b'\\' && byte != b'"' && byte != b'\''
                })
                || !content_type.contains('/')
            {
                return Err(ArtifactError::InvalidMetadata(
                    "content type must be a printable media type containing '/'".into(),
                ));
            }
        }
        self.retention.validate()
    }
}

/// Fully verified artifact delivered by a local service read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    pub metadata: crate::ArtifactMetadata,
    pub bytes: Vec<u8>,
}

impl Artifact {
    pub fn id(&self) -> &ArtifactId {
        &self.metadata.id
    }
}
