use crate::{ResourceRequest, RunnerError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkEnvelope {
    pub id: String,
    pub kind: String,
    pub attempt: u32,
    pub submitted_at_ms: u64,
    pub deadline_at_ms: u64,
    pub resources: ResourceRequest,
    pub input: Vec<u8>,
}

impl WorkEnvelope {
    pub fn new(
        id: impl Into<String>,
        kind: impl Into<String>,
        attempt: u32,
        submitted_at_ms: u64,
        deadline_at_ms: u64,
        resources: ResourceRequest,
        input: Vec<u8>,
    ) -> Result<Self, RunnerError> {
        let id = id.into();
        let kind = kind.into();
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(RunnerError::InvalidEnvelope(
                "id must be URL-safe and no longer than 128 bytes",
            ));
        }
        if kind.is_empty()
            || kind.len() > 128
            || !kind
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(RunnerError::InvalidEnvelope(
                "kind must be URL-safe and no longer than 128 bytes",
            ));
        }
        if attempt == 0 {
            return Err(RunnerError::InvalidEnvelope("attempt must start at one"));
        }
        if deadline_at_ms <= submitted_at_ms {
            return Err(RunnerError::InvalidEnvelope(
                "deadline must follow submission",
            ));
        }
        if input.len() > 64 << 20 {
            return Err(RunnerError::InvalidEnvelope("input exceeds 64 MiB"));
        }
        Ok(Self {
            id,
            kind,
            attempt,
            submitted_at_ms,
            deadline_at_ms,
            resources,
            input,
        })
    }
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms >= self.deadline_at_ms
    }
}
