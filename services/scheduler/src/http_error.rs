//! Transport error mapping for the scheduler control plane.
//!
//! A service must translate its domain errors into a documented transport format
//! rather than leaking raw internal strings as the sole machine-readable
//! contract. This module is the single place that assigns each
//! [`SchedulerError`] a stable HTTP status and machine-readable `code`, alongside
//! the human-readable `message` taken from the error's `Display`.

use crate::SchedulerError;
use crate::json::Json;

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
/// The mapping distinguishes caller mistakes (400), missing entities (404),
/// lifecycle and uniqueness conflicts (409), oversized checkpoints (413), server
/// faults (500), and unlinked transports (501) so clients can react without
/// parsing free-form messages.
pub fn classify(error: &SchedulerError) -> TransportError {
    match error {
        SchedulerError::InvalidConfig(_) => TransportError::new(400, "invalid_config"),
        SchedulerError::InvalidWorker(_) => TransportError::new(400, "invalid_worker"),
        SchedulerError::UnknownJob(_) => TransportError::new(404, "unknown_job"),
        SchedulerError::UnknownWorker(_) => TransportError::new(404, "unknown_worker"),
        SchedulerError::DuplicateJob(_) => TransportError::new(409, "duplicate_job"),
        SchedulerError::QueueFull { .. } => TransportError::new(409, "queue_full"),
        SchedulerError::StaleLease { .. } => TransportError::new(409, "stale_lease"),
        SchedulerError::LeaseExpired { .. } => TransportError::new(409, "lease_expired"),
        SchedulerError::InvalidTransition { .. } => TransportError::new(409, "invalid_transition"),
        SchedulerError::CheckpointTooLarge { .. } => {
            TransportError::new(413, "checkpoint_too_large")
        }
        SchedulerError::CorruptCheckpoint(_) => TransportError::new(500, "corrupt_checkpoint"),
        SchedulerError::Store(_) => TransportError::new(500, "store_error"),
        SchedulerError::UnsupportedTransport(_) => {
            TransportError::new(501, "unsupported_transport")
        }
    }
}

/// Builds the machine-readable JSON error body advertised to clients.
pub fn body(error: &SchedulerError) -> Json {
    let classified = classify(error);
    Json::Object(vec![
        ("code".into(), Json::string(classified.code)),
        ("message".into(), Json::string(error.to_string())),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_domain_errors_to_documented_statuses() {
        assert_eq!(classify(&SchedulerError::InvalidConfig("x".into())).status, 400);
        assert_eq!(classify(&SchedulerError::InvalidWorker("x".into())).status, 400);
        assert_eq!(classify(&SchedulerError::UnknownJob("j".into())).status, 404);
        assert_eq!(classify(&SchedulerError::UnknownWorker("w".into())).status, 404);
        assert_eq!(classify(&SchedulerError::DuplicateJob("j".into())).status, 409);
        assert_eq!(classify(&SchedulerError::QueueFull { limit: 1 }).status, 409);
        assert_eq!(classify(&SchedulerError::StaleLease { job_id: "j".into() }).status, 409);
        assert_eq!(classify(&SchedulerError::LeaseExpired { job_id: "j".into() }).status, 409);
        assert_eq!(
            classify(&SchedulerError::InvalidTransition { job_id: "j".into(), state: "completed" })
                .status,
            409
        );
        assert_eq!(
            classify(&SchedulerError::CheckpointTooLarge { actual: 2, limit: 1 }).status,
            413
        );
        assert_eq!(classify(&SchedulerError::CorruptCheckpoint("x".into())).status, 500);
        assert_eq!(classify(&SchedulerError::UnsupportedTransport("x")).status, 501);
    }

    #[test]
    fn body_carries_a_stable_code_and_message() {
        let json = body(&SchedulerError::UnknownJob("job-9".into())).render();
        assert!(json.contains("\"code\":\"unknown_job\""));
        assert!(json.contains("job-9"));
    }
}
