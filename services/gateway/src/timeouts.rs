//! Upstream connect and read timeouts over `std::net`.
//!
//! A reverse proxy that blocks forever on a stuck upstream is a denial-of-service
//! vector. This module centralises timeout application: a bounded connect using
//! `TcpStream::connect_timeout`, and read/write deadlines via `set_read_timeout`
//! / `set_write_timeout`. Address resolution is std-only through
//! `ToSocketAddrs`.

use crate::errors::GatewayError;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Opens a connection to `addr`, failing fast if it cannot be established within
/// `timeout`, and arms read/write deadlines on the returned stream.
pub fn connect(addr: &str, timeout: Duration) -> Result<TcpStream, GatewayError> {
    let mut resolved = addr
        .to_socket_addrs()
        .map_err(|error| GatewayError::UpstreamUnavailable(format!("resolve {addr}: {error}")))?;
    let socket = resolved
        .next()
        .ok_or_else(|| GatewayError::UpstreamUnavailable(format!("no address for {addr}")))?;

    let stream = TcpStream::connect_timeout(&socket, timeout)
        .map_err(|error| GatewayError::UpstreamUnavailable(format!("connect {addr}: {error}")))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| GatewayError::UpstreamUnavailable(error.to_string()))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| GatewayError::UpstreamUnavailable(error.to_string()))?;
    Ok(stream)
}

/// Classifies an I/O error observed while talking to the upstream, mapping the
/// timeout kinds to [`GatewayError::UpstreamTimeout`] and the rest to an
/// unavailable upstream.
pub fn classify_io(error: &std::io::Error) -> GatewayError {
    match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            GatewayError::UpstreamTimeout
        }
        _ => GatewayError::UpstreamUnavailable(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_refused_maps_to_unavailable() {
        // Port 1 on loopback is not listening; connect must fail promptly.
        let result = connect("127.0.0.1:1", Duration::from_millis(200));
        assert!(matches!(result, Err(GatewayError::UpstreamUnavailable(_))));
    }

    #[test]
    fn timed_out_io_is_classified() {
        let error = std::io::Error::new(std::io::ErrorKind::TimedOut, "slow");
        assert!(matches!(classify_io(&error), GatewayError::UpstreamTimeout));
    }
}
