//! Bounded streaming copy for large artifact download bodies.
//!
//! When the gateway relays a large artifact GET from the backend, it copies the
//! body in fixed-size chunks rather than materialising it in one allocation, and
//! it enforces an absolute ceiling so a misbehaving or compromised upstream can
//! never exhaust gateway memory. The same primitive backs upload passthrough.

use crate::errors::GatewayError;
use std::io::{Read, Write};

/// The size of each copy chunk. Small enough to bound peak buffer, large enough
/// to keep syscall overhead negligible for multi-megabyte artifacts.
pub const CHUNK_BYTES: usize = 32 * 1024;

/// Copies bytes from `reader` to `writer` in bounded chunks.
///
/// Returns the total number of bytes copied, or [`GatewayError::PayloadTooLarge`]
/// if the stream would exceed `max_bytes`. The write side is flushed by the
/// caller.
pub fn stream_copy<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    max_bytes: usize,
) -> Result<usize, GatewayError> {
    let mut buffer = [0u8; CHUNK_BYTES];
    let mut total = 0usize;
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| GatewayError::UpstreamUnavailable(error.to_string()))?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read)
            .filter(|sum| *sum <= max_bytes)
            .ok_or(GatewayError::PayloadTooLarge)?;
        writer
            .write_all(&buffer[..read])
            .map_err(|error| GatewayError::UpstreamUnavailable(error.to_string()))?;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_within_the_ceiling() {
        let source = vec![7u8; 100 * 1024];
        let mut sink = Vec::new();
        let copied = stream_copy(&mut &source[..], &mut sink, 1024 * 1024).unwrap();
        assert_eq!(copied, source.len());
        assert_eq!(sink, source);
    }

    #[test]
    fn rejects_a_stream_over_the_ceiling() {
        let source = vec![0u8; 64 * 1024];
        let mut sink = Vec::new();
        let result = stream_copy(&mut &source[..], &mut sink, 4 * 1024);
        assert!(matches!(result, Err(GatewayError::PayloadTooLarge)));
    }
}
