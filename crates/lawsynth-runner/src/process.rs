use crate::{CancellationToken, Checkpoint, RunnerError, WorkEnvelope};

/// Process implementations must cooperatively check cancellation and can emit
/// a checkpoint only after their previous state is complete and durable.
pub trait WorkProcess {
    type Output;
    fn execute(
        &mut self,
        envelope: &WorkEnvelope,
        cancellation: &CancellationToken,
    ) -> Result<Self::Output, RunnerError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessRecord {
    pub work_id: String,
    pub checkpoints: Vec<Checkpoint>,
}

impl ProcessRecord {
    pub fn new(work_id: impl Into<String>) -> Self {
        Self { work_id: work_id.into(), checkpoints: Vec::new() }
    }
    pub fn record_checkpoint(&mut self, checkpoint: Checkpoint) -> Result<(), RunnerError> {
        checkpoint.verify()?;
        if let Some(previous) = self.checkpoints.last()
            && checkpoint.sequence <= previous.sequence
        {
            return Err(RunnerError::CheckpointRejected("sequence must increase strictly"));
        }
        self.checkpoints.push(checkpoint);
        Ok(())
    }
    pub fn latest_checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }
}
