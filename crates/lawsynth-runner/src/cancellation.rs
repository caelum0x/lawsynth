use std::sync::{Arc, Mutex};

use crate::RunnerError;

#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<Mutex<Option<String>>>);

impl CancellationToken {
    pub fn cancel(&self, reason: impl Into<String>) -> Result<(), RunnerError> {
        let reason = reason.into();
        if reason.trim().is_empty() || reason.len() > 512 {
            return Err(RunnerError::InvalidEnvelope(
                "cancellation reason must contain 1..=512 bytes",
            ));
        }
        let mut state = self.0.lock().expect("cancellation mutex poisoned");
        if state.is_none() {
            *state = Some(reason);
        }
        Ok(())
    }
    pub fn reason(&self) -> Option<String> {
        self.0.lock().expect("cancellation mutex poisoned").clone()
    }
    pub fn check(&self) -> Result<(), RunnerError> {
        self.reason()
            .map_or(Ok(()), |reason| Err(RunnerError::Cancelled { reason }))
    }
    pub fn is_cancelled(&self) -> bool {
        self.reason().is_some()
    }
}
