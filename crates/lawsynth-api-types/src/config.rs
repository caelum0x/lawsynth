use crate::ApiValidationError;

/// Limits applied by an API boundary before a request reaches the engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApiLimits {
    pub maximum_page_size: u32,
    pub maximum_artifact_bytes: u64,
    pub maximum_event_payload_bytes: u32,
}

impl ApiLimits {
    pub const DEFAULT: Self = Self {
        maximum_page_size: 250,
        maximum_artifact_bytes: 1 << 30,
        maximum_event_payload_bytes: 64 << 10,
    };

    pub fn new(
        maximum_page_size: u32,
        maximum_artifact_bytes: u64,
        maximum_event_payload_bytes: u32,
    ) -> Result<Self, ApiValidationError> {
        if !(1..=10_000).contains(&maximum_page_size) {
            return Err(ApiValidationError::OutOfRange {
                field: "maximum_page_size",
                minimum: 1,
                maximum: 10_000,
            });
        }
        if maximum_artifact_bytes == 0 {
            return Err(ApiValidationError::Invalid {
                field: "maximum_artifact_bytes",
                reason: "must be positive",
            });
        }
        if maximum_event_payload_bytes == 0 {
            return Err(ApiValidationError::Invalid {
                field: "maximum_event_payload_bytes",
                reason: "must be positive",
            });
        }
        Ok(Self {
            maximum_page_size,
            maximum_artifact_bytes,
            maximum_event_payload_bytes,
        })
    }
}
