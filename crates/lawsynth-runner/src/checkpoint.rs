use crate::RunnerError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub sequence: u64,
    pub created_at_ms: u64,
    pub payload: Vec<u8>,
    pub checksum: u64,
}

impl Checkpoint {
    pub fn new(
        sequence: u64,
        created_at_ms: u64,
        payload: Vec<u8>,
        maximum_payload_bytes: usize,
    ) -> Result<Self, RunnerError> {
        if payload.is_empty() {
            return Err(RunnerError::CheckpointRejected("payload must not be empty"));
        }
        if payload.len() > maximum_payload_bytes {
            return Err(RunnerError::CheckpointRejected(
                "payload exceeds configured size limit",
            ));
        }
        let checksum = checksum(&payload);
        Ok(Self {
            sequence,
            created_at_ms,
            payload,
            checksum,
        })
    }
    pub fn verify(&self) -> Result<(), RunnerError> {
        if checksum(&self.payload) != self.checksum {
            return Err(RunnerError::CheckpointRejected(
                "checksum does not match payload",
            ));
        }
        Ok(())
    }
}

fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |state, byte| {
        (state ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
