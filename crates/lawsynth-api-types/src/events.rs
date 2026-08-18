use crate::{ApiValidationError, ProjectId, RunId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventKind {
    RunQueued,
    RunStarted,
    Progress,
    RunSucceeded,
    RunFailed,
    RunCancelled,
    ArtifactCreated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiEvent {
    pub sequence: u64,
    pub occurred_at_ms: u64,
    pub project_id: ProjectId,
    pub run_id: Option<RunId>,
    pub kind: EventKind,
    pub payload: String,
}

impl ApiEvent {
    pub fn new(
        sequence: u64,
        occurred_at_ms: u64,
        project_id: ProjectId,
        run_id: Option<RunId>,
        kind: EventKind,
        payload: impl Into<String>,
        payload_limit: u32,
    ) -> Result<Self, ApiValidationError> {
        let payload = payload.into();
        if payload.len() > payload_limit as usize {
            return Err(ApiValidationError::TooLong {
                field: "payload",
                maximum: payload_limit as usize,
            });
        }
        if payload.contains('\0') {
            return Err(ApiValidationError::Invalid {
                field: "payload",
                reason: "NUL is not allowed",
            });
        }
        if matches!(
            kind,
            EventKind::RunQueued
                | EventKind::RunStarted
                | EventKind::Progress
                | EventKind::RunSucceeded
                | EventKind::RunFailed
                | EventKind::RunCancelled
        ) && run_id.is_none()
        {
            return Err(ApiValidationError::Inconsistent { reason: "run event requires run_id" });
        }
        Ok(Self { sequence, occurred_at_ms, project_id, run_id, kind, payload })
    }
}

pub fn validate_event_stream(events: &[ApiEvent]) -> Result<(), ApiValidationError> {
    for pair in events.windows(2) {
        if pair[1].sequence <= pair[0].sequence {
            return Err(ApiValidationError::Invalid {
                field: "events.sequence",
                reason: "must increase strictly",
            });
        }
        if pair[1].occurred_at_ms < pair[0].occurred_at_ms {
            return Err(ApiValidationError::Invalid {
                field: "events.occurred_at_ms",
                reason: "must not go backwards",
            });
        }
    }
    Ok(())
}
