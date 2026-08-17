//! Transport error mapping for the worker's status/health surface.
//!
//! This is the single place that assigns each [`WorkerError`] a stable HTTP
//! status and a machine-readable `code`, alongside a human-readable `message`.
//! The status surface never accepts executable jobs, so most caller-facing
//! failures are validation (`400`), lookup (`404`), admission pressure (`429`),
//! or lifecycle conflicts (`409`); genuine execution and storage faults are
//! reported as server conditions (`5xx`).

use crate::WorkerError;
use crate::json::Json;
use lawsynth_runner::RunnerError;
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
/// Caller mistakes (4xx) are separated from admission pressure (429), lifecycle
/// conflicts (409), unimplemented transports (501), and internal faults (5xx)
/// so clients can react without parsing free-form messages.
pub fn classify(error: &WorkerError) -> TransportError {
    match error {
        // Job validation is a caller-correctable input error.
        WorkerError::InvalidJob(_) => TransportError::new(400, "invalid_job"),
        // A configuration fault is a server condition, not a client mistake.
        WorkerError::InvalidConfig(_) => TransportError::new(500, "invalid_config"),
        // The job's deadline has already passed: a lifecycle conflict, not a retry.
        WorkerError::DeadlineExceeded { .. } => TransportError::new(409, "deadline_exceeded"),
        // A durable record already exists for this id: a lifecycle conflict.
        WorkerError::DuplicateJob(_) => TransportError::new(409, "duplicate_job"),
        WorkerError::Cancelled(_) => TransportError::new(409, "cancelled"),
        WorkerError::Runner(runner) => classify_runner(runner),
        // Discovery and simulation failures are internal execution faults.
        WorkerError::Discovery(_) => TransportError::new(500, "discovery_failed"),
        WorkerError::Simulation(_) => TransportError::new(500, "simulation_failed"),
        WorkerError::Store(store) => classify_store(store),
        // A corrupt persisted checkpoint is a storage integrity fault.
        WorkerError::CorruptCheckpoint(_) => TransportError::new(500, "corrupt_checkpoint"),
        WorkerError::UnsupportedTransport(_) => TransportError::new(501, "unsupported_transport"),
    }
}

fn classify_runner(error: &RunnerError) -> TransportError {
    match error {
        RunnerError::InvalidConfig(_) => TransportError::new(500, "invalid_config"),
        RunnerError::InvalidEnvelope(_) => TransportError::new(400, "invalid_envelope"),
        // Admission was refused because the shared capacity budget is exhausted.
        RunnerError::CapacityExceeded { .. } => TransportError::new(429, "capacity_exceeded"),
        RunnerError::Cancelled { .. } => TransportError::new(409, "cancelled"),
        RunnerError::CheckpointRejected(_) => TransportError::new(500, "checkpoint_rejected"),
        RunnerError::ProcessFailed(_) => TransportError::new(500, "process_failed"),
    }
}

fn classify_store(error: &StoreError) -> TransportError {
    match error {
        StoreError::InvalidKey(_) => TransportError::new(400, "invalid_key"),
        StoreError::NotFound(_) => TransportError::new(404, "not_found"),
        StoreError::ObjectTooLarge { .. } => TransportError::new(413, "object_too_large"),
        StoreError::InvalidPart(_) => TransportError::new(400, "invalid_part"),
        StoreError::Unsupported(_) => TransportError::new(501, "unsupported"),
        StoreError::Io(_) => TransportError::new(500, "io_error"),
    }
}

/// Builds the machine-readable JSON error body advertised to clients.
pub fn body(error: &WorkerError) -> Json {
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
        assert_eq!(classify(&WorkerError::InvalidJob("bad".into())).status, 400);
        assert_eq!(
            classify(&WorkerError::DeadlineExceeded { job_id: "j".into(), deadline_at_ms: 1 })
                .status,
            409
        );
        assert_eq!(classify(&WorkerError::DuplicateJob("j".into())).status, 409);
        assert_eq!(classify(&WorkerError::Cancelled("stop".into())).status, 409);
        assert_eq!(
            classify(&WorkerError::Runner(RunnerError::CapacityExceeded {
                requested: 2,
                available: 1
            }))
            .status,
            429
        );
        assert_eq!(classify(&WorkerError::Runner(RunnerError::InvalidEnvelope("x"))).status, 400);
        assert_eq!(classify(&WorkerError::UnsupportedTransport("no queue")).status, 501);
        assert_eq!(classify(&WorkerError::CorruptCheckpoint("x".into())).status, 500);
        assert_eq!(classify(&WorkerError::Store(StoreError::NotFound("k".into()))).status, 404);
    }

    #[test]
    fn body_carries_a_stable_code_and_message() {
        let json = body(&WorkerError::InvalidJob("discovery requires samples".into())).render();
        assert!(json.contains("\"code\":\"invalid_job\""));
        assert!(json.contains("discovery requires samples"));
    }
}
