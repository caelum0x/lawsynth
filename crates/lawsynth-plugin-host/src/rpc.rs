use crate::HostError;
use lawsynth_plugin_api::{Frame, MAX_FRAME_BYTES};
use std::io::{Read, Write};

pub fn write_frame(writer: &mut impl Write, frame: &Frame) -> Result<(), HostError> {
    let bytes = frame.encode()?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}
pub fn read_frame(reader: &mut impl Read) -> Result<Frame, HostError> {
    let mut prefix = [0; 4];
    reader.read_exact(&mut prefix)?;
    let declared = u32::from_be_bytes(prefix) as usize;
    if declared > MAX_FRAME_BYTES {
        return Err(HostError::Resource("incoming frame exceeds limit".into()));
    }
    let mut bytes = Vec::with_capacity(declared + 4);
    bytes.extend_from_slice(&prefix);
    bytes.resize(declared + 4, 0);
    reader.read_exact(&mut bytes[4..])?;
    Ok(Frame::decode(&bytes)?)
}
/// A typed framed transport for a child process's stdin/stdout pipes.
pub struct RpcChannel<R, W> {
    reader: R,
    writer: W,
}
impl<R: Read, W: Write> RpcChannel<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }
    pub fn send(&mut self, frame: &Frame) -> Result<(), HostError> {
        write_frame(&mut self.writer, frame)
    }
    pub fn receive(&mut self) -> Result<Frame, HostError> {
        read_frame(&mut self.reader)
    }
    pub fn into_inner(self) -> (R, W) {
        (self.reader, self.writer)
    }
}
