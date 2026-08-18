use crate::PluginError;

/// Host-enforced per-plugin execution limits. Zero values are rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceLimits {
    pub max_cpu_millis: u64,
    pub max_memory_bytes: u64,
    pub max_output_bytes: u64,
    pub max_requests: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_millis: 30_000,
            max_memory_bytes: 256 * 1024 * 1024,
            max_output_bytes: 16 * 1024 * 1024,
            max_requests: 1_000,
        }
    }
}

impl ResourceLimits {
    pub fn validate(self) -> Result<Self, PluginError> {
        if self.max_cpu_millis == 0
            || self.max_memory_bytes == 0
            || self.max_output_bytes == 0
            || self.max_requests == 0
        {
            return Err(PluginError::InvalidLimits(
                "all resource limits must be greater than zero".into(),
            ));
        }
        if self.max_memory_bytes > 8 * 1024 * 1024 * 1024 {
            return Err(PluginError::InvalidLimits("memory limit exceeds host maximum".into()));
        }
        if self.max_output_bytes > self.max_memory_bytes {
            return Err(PluginError::InvalidLimits(
                "output limit cannot exceed memory limit".into(),
            ));
        }
        Ok(self)
    }

    pub fn permits(self, requested: Self) -> bool {
        requested.max_cpu_millis <= self.max_cpu_millis
            && requested.max_memory_bytes <= self.max_memory_bytes
            && requested.max_output_bytes <= self.max_output_bytes
            && requested.max_requests <= self.max_requests
    }
}
