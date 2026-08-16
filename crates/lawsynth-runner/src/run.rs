use crate::{CancellationToken, ResourceLimiter, RunnerError, WorkEnvelope, WorkProcess};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionStatus {
    Succeeded,
    Failed,
    Cancelled,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionReport {
    pub work_id: String,
    pub status: ExecutionStatus,
    pub message: Option<String>,
}

/// Runs one work item after atomically admitting and finally releasing its resources.
pub fn execute<P: WorkProcess>(
    limiter: &mut ResourceLimiter,
    envelope: &WorkEnvelope,
    process: &mut P,
    cancellation: &CancellationToken,
) -> Result<P::Output, RunnerError> {
    cancellation.check()?;
    limiter.reserve(envelope.resources)?;
    let result = process.execute(envelope, cancellation);
    limiter.release(envelope.resources)?;
    result
}

pub fn classify_result<T>(
    work_id: impl Into<String>,
    result: &Result<T, RunnerError>,
) -> ExecutionReport {
    let (status, message) = match result {
        Ok(_) => (ExecutionStatus::Succeeded, None),
        Err(RunnerError::Cancelled { reason }) => {
            (ExecutionStatus::Cancelled, Some(reason.clone()))
        }
        Err(RunnerError::CapacityExceeded { .. } | RunnerError::InvalidEnvelope(_)) => (
            ExecutionStatus::Rejected,
            Some(result.as_ref().err().expect("error branch").to_string()),
        ),
        Err(error) => (ExecutionStatus::Failed, Some(error.to_string())),
    };
    ExecutionReport {
        work_id: work_id.into(),
        status,
        message,
    }
}
