use crate::RunnerError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceRequest {
    pub cpu_millis: u32,
    pub memory_bytes: u64,
    pub disk_bytes: u64,
}

impl ResourceRequest {
    pub fn new(cpu_millis: u32, memory_bytes: u64, disk_bytes: u64) -> Result<Self, RunnerError> {
        if cpu_millis == 0 {
            return Err(RunnerError::InvalidEnvelope("cpu_millis must be positive"));
        }
        if memory_bytes == 0 {
            return Err(RunnerError::InvalidEnvelope("memory_bytes must be positive"));
        }
        Ok(Self { cpu_millis, memory_bytes, disk_bytes })
    }

    pub const fn fits_within(self, capacity: Self) -> bool {
        self.cpu_millis <= capacity.cpu_millis
            && self.memory_bytes <= capacity.memory_bytes
            && self.disk_bytes <= capacity.disk_bytes
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            cpu_millis: self.cpu_millis.checked_add(other.cpu_millis)?,
            memory_bytes: self.memory_bytes.checked_add(other.memory_bytes)?,
            disk_bytes: self.disk_bytes.checked_add(other.disk_bytes)?,
        })
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        Some(Self {
            cpu_millis: self.cpu_millis.checked_sub(other.cpu_millis)?,
            memory_bytes: self.memory_bytes.checked_sub(other.memory_bytes)?,
            disk_bytes: self.disk_bytes.checked_sub(other.disk_bytes)?,
        })
    }
}
