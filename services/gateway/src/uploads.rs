//! Bounded streaming passthrough for large upload request bodies.
//!
//! Uploads flow client -> gateway -> backend. This module reuses the bounded
//! chunked copy from [`crate::downloads`] so both directions share one audited
//! implementation of "copy at most N bytes, in fixed chunks". Keeping the upload
//! seam named separately documents the intent at call sites and leaves room for
//! direction-specific policy without duplicating the copy loop.

use crate::downloads::stream_copy;
use crate::errors::GatewayError;
use std::io::{Read, Write};

/// Streams an upload body from `reader` to `writer`, capped at `max_bytes`.
///
/// Returns the number of bytes forwarded, or [`GatewayError::PayloadTooLarge`]
/// if the body would exceed the configured maximum.
pub fn passthrough<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    max_bytes: usize,
) -> Result<usize, GatewayError> {
    stream_copy(reader, writer, max_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwards_a_bounded_upload() {
        let body = vec![3u8; 50 * 1024];
        let mut sink = Vec::new();
        let forwarded = passthrough(&mut &body[..], &mut sink, 1024 * 1024).unwrap();
        assert_eq!(forwarded, body.len());
        assert_eq!(sink, body);
    }

    #[test]
    fn rejects_an_oversized_upload() {
        let body = vec![1u8; 16 * 1024];
        let mut sink = Vec::new();
        assert!(matches!(
            passthrough(&mut &body[..], &mut sink, 1024),
            Err(GatewayError::PayloadTooLarge)
        ));
    }
}
