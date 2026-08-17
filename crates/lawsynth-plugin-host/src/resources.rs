use crate::HostError;
use lawsynth_plugin_api::ResourceLimits;
use std::time::{Duration, Instant};

/// Accounting for host-observable request, output, and elapsed-time budgets.
#[derive(Debug)]
pub struct ResourceMeter {
    limits: ResourceLimits,
    started: Instant,
    requests: u32,
    output_bytes: u64,
}
impl ResourceMeter {
    pub fn new(limits: ResourceLimits) -> Result<Self, HostError> {
        limits.validate()?;
        Ok(Self { limits, started: Instant::now(), requests: 0, output_bytes: 0 })
    }
    pub fn begin_request(&mut self) -> Result<(), HostError> {
        self.check_time()?;
        self.requests = self
            .requests
            .checked_add(1)
            .ok_or_else(|| HostError::Resource("request counter overflow".into()))?;
        if self.requests > self.limits.max_requests {
            return Err(HostError::Resource("request limit exceeded".into()));
        }
        Ok(())
    }
    pub fn record_output(&mut self, bytes: usize) -> Result<(), HostError> {
        self.check_time()?;
        self.output_bytes = self
            .output_bytes
            .checked_add(
                u64::try_from(bytes)
                    .map_err(|_| HostError::Resource("output size overflow".into()))?,
            )
            .ok_or_else(|| HostError::Resource("output counter overflow".into()))?;
        if self.output_bytes > self.limits.max_output_bytes {
            return Err(HostError::Resource("output byte limit exceeded".into()));
        }
        Ok(())
    }
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
    fn check_time(&self) -> Result<(), HostError> {
        if self.elapsed() > Duration::from_millis(self.limits.max_cpu_millis) {
            Err(HostError::Resource("wall-clock execution limit exceeded".into()))
        } else {
            Ok(())
        }
    }
}
