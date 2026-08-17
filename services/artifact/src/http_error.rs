//! Transport error mapping.
//!
//! `specs/service-api/errors.md` requires a service to map local validation and
//! dependency failures into a documented transport error format and forbids
//! exposing raw internal strings as the sole machine-readable contract. This
//! module is the single place that assigns each [`ArtifactError`] a stable HTTP
//! status and machine-readable `code`, alongside a human-readable `message`.

use crate::ArtifactError;
use crate::json::Json;
use lawsynth_store::StoreError;

/// A documented transport failure: an HTTP status plus a stable machine code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportError {
    pub status: u16,
    pub code: &'static str,
}

impl TransportError {
    pub const fn new(status: u16, code: &'static str) -> Self {
        Self { status, code }
    }
}

/// Maps a domain error to its documented transport status and stable code.
///
/// The mapping intentionally distinguishes caller mistakes (4xx) from storage
/// pressure (507), payload limits (413), and internal corruption (5xx) so that
/// clients can react without parsing free-form messages.
pub fn classify(error: &ArtifactError) -> TransportError {
    match error {
        ArtifactError::InvalidArtifactId(_) => TransportError::new(400, "invalid_artifact_id"),
        ArtifactError::InvalidMetadata(_) => TransportError::new(400, "invalid_metadata"),
        ArtifactError::InvalidUpload(_) => TransportError::new(400, "invalid_upload"),
        ArtifactError::NotFound(_) => TransportError::new(404, "not_found"),
        ArtifactError::Expired(_) => TransportError::new(410, "expired"),
        ArtifactError::CapacityExceeded { .. } => TransportError::new(507, "capacity_exceeded"),
        // A configuration fault is a server condition, not a client-correctable one.
        ArtifactError::InvalidConfig(_) => TransportError::new(500, "invalid_config"),
        // Checksum and metadata corruption indicate a storage integrity fault.
        ArtifactError::ChecksumMismatch { .. } => TransportError::new(500, "checksum_mismatch"),
        ArtifactError::CorruptMetadata(_) => TransportError::new(500, "corrupt_metadata"),
        ArtifactError::Store(store) => classify_store(store),
    }
}

fn classify_store(error: &StoreError) -> TransportError {
    match error {
        StoreError::InvalidKey(_) => TransportError::new(400, "invalid_key"),
        StoreError::InvalidPart(_) => TransportError::new(400, "invalid_part"),
        StoreError::NotFound(_) => TransportError::new(404, "not_found"),
        StoreError::ObjectTooLarge { .. } => TransportError::new(413, "object_too_large"),
        StoreError::Unsupported(_) => TransportError::new(501, "unsupported"),
        StoreError::Io(_) => TransportError::new(500, "io_error"),
    }
}

/// Builds the machine-readable JSON error body advertised to clients.
pub fn body(error: &ArtifactError) -> Json {
    let classified = classify(error);
    Json::Object(vec![
        ("code".into(), Json::string(classified.code)),
        ("message".into(), Json::string(error.to_string())),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtifactId;
    use crate::checksum::sha256;

    #[test]
    fn maps_domain_errors_to_documented_statuses() {
        assert_eq!(classify(&ArtifactError::InvalidUpload("x".into())).status, 400);
        assert_eq!(
            classify(&ArtifactError::NotFound(ArtifactId::new(sha256(b"a")).unwrap())).status,
            404
        );
        assert_eq!(
            classify(&ArtifactError::Expired(ArtifactId::new(sha256(b"a")).unwrap())).status,
            410
        );
        assert_eq!(
            classify(&ArtifactError::CapacityExceeded { requested: 2, available: 1 }).status,
            507
        );
        assert_eq!(
            classify(&ArtifactError::Store(StoreError::ObjectTooLarge { actual: 2, limit: 1 }))
                .status,
            413
        );
    }

    #[test]
    fn body_carries_a_stable_code_and_message() {
        let json = body(&ArtifactError::InvalidUpload("unknown upload".into())).render();
        assert!(json.contains("\"code\":\"invalid_upload\""));
        assert!(json.contains("invalid multipart upload"));
    }
}
