use crate::{PROTOCOL_VERSION, PluginError};

/// Protocol configuration negotiated before an extension receives data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolConfig {
    pub version: u16,
    pub max_frame_bytes: usize,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self { version: PROTOCOL_VERSION, max_frame_bytes: crate::MAX_FRAME_BYTES }
    }
}

impl ProtocolConfig {
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.version != PROTOCOL_VERSION {
            return Err(PluginError::Protocol(format!(
                "protocol version {} is not supported",
                self.version
            )));
        }
        if self.max_frame_bytes == 0 || self.max_frame_bytes > crate::MAX_FRAME_BYTES {
            return Err(PluginError::Protocol("invalid maximum frame size".into()));
        }
        Ok(())
    }
}
