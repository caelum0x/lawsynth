use crate::{ResourceRequest, RunnerError};

/// Single-threaded admission controller. Put it behind a mutex when shared.
#[derive(Clone, Debug)]
pub struct ResourceLimiter {
    capacity: ResourceRequest,
    reserved: ResourceRequest,
}

impl ResourceLimiter {
    pub fn new(capacity: ResourceRequest) -> Self {
        Self {
            capacity,
            reserved: ResourceRequest { cpu_millis: 0, memory_bytes: 0, disk_bytes: 0 },
        }
    }
    pub const fn capacity(&self) -> ResourceRequest {
        self.capacity
    }
    pub const fn reserved(&self) -> ResourceRequest {
        self.reserved
    }
    pub fn available(&self) -> ResourceRequest {
        self.capacity.checked_sub(self.reserved).expect("reservation cannot exceed capacity")
    }
    pub fn reserve(&mut self, request: ResourceRequest) -> Result<(), RunnerError> {
        if !request.fits_within(self.available()) {
            return Err(RunnerError::CapacityExceeded {
                requested: request.memory_bytes,
                available: self.available().memory_bytes,
            });
        }
        self.reserved = self
            .reserved
            .checked_add(request)
            .expect("validated resource reservation must not overflow");
        Ok(())
    }
    pub fn release(&mut self, request: ResourceRequest) -> Result<(), RunnerError> {
        self.reserved = self
            .reserved
            .checked_sub(request)
            .ok_or(RunnerError::InvalidEnvelope("released resources were not reserved"))?;
        Ok(())
    }
}
