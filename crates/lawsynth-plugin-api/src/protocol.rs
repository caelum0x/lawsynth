use crate::PluginError;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
const HEADER_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FrameKind {
    Hello = 1,
    Request = 2,
    Response = 3,
    Error = 4,
    Shutdown = 5,
}
impl TryFrom<u8> for FrameKind {
    type Error = PluginError;
    fn try_from(v: u8) -> Result<Self, PluginError> {
        match v {
            1 => Ok(Self::Hello),
            2 => Ok(Self::Request),
            3 => Ok(Self::Response),
            4 => Ok(Self::Error),
            5 => Ok(Self::Shutdown),
            _ => Err(PluginError::Protocol(format!("unknown frame kind {v}"))),
        }
    }
}

/// Length-delimited binary message. Bytes 0..4 encode total bytes after the
/// prefix; all multi-byte fields are big endian to make captures portable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Frame {
    pub kind: FrameKind,
    pub request_id: u64,
    pub payload: Vec<u8>,
}
impl Frame {
    pub fn new(kind: FrameKind, request_id: u64, payload: Vec<u8>) -> Result<Self, PluginError> {
        let frame = Self {
            kind,
            request_id,
            payload,
        };
        frame.validate()?;
        Ok(frame)
    }
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.payload.len() > MAX_FRAME_BYTES - HEADER_BYTES {
            Err(PluginError::Protocol("frame payload exceeds limit".into()))
        } else {
            Ok(())
        }
    }
    pub fn encode(&self) -> Result<Vec<u8>, PluginError> {
        self.validate()?;
        let body_len = HEADER_BYTES + self.payload.len();
        let body_u32 = u32::try_from(body_len)
            .map_err(|_| PluginError::Protocol("frame is too large".into()))?;
        let mut out = Vec::with_capacity(4 + body_len);
        out.extend_from_slice(&body_u32.to_be_bytes());
        out.extend_from_slice(&PROTOCOL_VERSION.to_be_bytes());
        out.push(self.kind as u8);
        out.push(0);
        out.extend_from_slice(&self.request_id.to_be_bytes());
        out.extend_from_slice(&self.payload);
        Ok(out)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, PluginError> {
        if bytes.len() < 4 + HEADER_BYTES {
            return Err(PluginError::Protocol("truncated frame".into()));
        }
        let declared = u32::from_be_bytes(bytes[0..4].try_into().expect("fixed slice")) as usize;
        if declared > MAX_FRAME_BYTES || declared + 4 != bytes.len() {
            return Err(PluginError::Protocol("invalid frame length".into()));
        }
        let version = u16::from_be_bytes(bytes[4..6].try_into().expect("fixed slice"));
        if version != PROTOCOL_VERSION {
            return Err(PluginError::Protocol(format!(
                "unsupported protocol version {version}"
            )));
        }
        if bytes[7] != 0 {
            return Err(PluginError::Protocol(
                "reserved frame byte must be zero".into(),
            ));
        }
        let kind = FrameKind::try_from(bytes[6])?;
        let request_id = u64::from_be_bytes(bytes[8..16].try_into().expect("fixed slice"));
        Self::new(kind, request_id, bytes[16..].to_vec())
    }
}
