use crate::{ApiValidationError, ProjectId, project::validate_identifier};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RunId(String);

impl RunId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ApiValidationError> {
        let value = value.into();
        validate_identifier("run_id", &value, 128)?;
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl RunStatus {
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Queued, Self::Running | Self::Cancelled)
                | (Self::Running, Self::Succeeded | Self::Failed | Self::Cancelled)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunSummary {
    pub id: RunId,
    pub project_id: ProjectId,
    pub status: RunStatus,
    pub created_at_ms: u64,
    pub finished_at_ms: Option<u64>,
}

impl RunSummary {
    pub fn new(
        id: RunId,
        project_id: ProjectId,
        status: RunStatus,
        created_at_ms: u64,
        finished_at_ms: Option<u64>,
    ) -> Result<Self, ApiValidationError> {
        if status.is_terminal() != finished_at_ms.is_some() {
            return Err(ApiValidationError::Inconsistent {
                reason: "terminal status and finished_at_ms must agree",
            });
        }
        if let Some(finished_at_ms) = finished_at_ms
            && finished_at_ms < created_at_ms
        {
            return Err(ApiValidationError::Invalid {
                field: "finished_at_ms",
                reason: "cannot precede creation",
            });
        }
        Ok(Self { id, project_id, status, created_at_ms, finished_at_ms })
    }
}
