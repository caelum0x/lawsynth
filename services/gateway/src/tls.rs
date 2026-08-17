//! The honest TLS-termination seam.
//!
//! The Rust standard library ships no TLS implementation, and this crate is
//! deliberately dependency-free. Rather than pretend to terminate TLS, the
//! gateway declares *where* termination is expected to happen and refuses to
//! fabricate a handshake. In the reference deployment an external edge (the
//! Caddy service in `compose`) terminates TLS and forwards cleartext HTTP/1.1 to
//! this gateway on the loopback interface.
//!
//! This mirrors the repository's honest-boundary style used elsewhere (for
//! example the artifact service refusing to simulate a remote object store).

use std::fmt;

/// How TLS is expected to be handled for a given gateway deployment.
///
/// There is intentionally no `Native` / `Terminating` variant: the gateway does
/// not and cannot terminate TLS itself with std alone.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TlsMode {
    /// Plain HTTP with no TLS anywhere. Suitable only for local development or a
    /// trusted private network.
    Disabled,
    /// TLS is terminated by an external terminator (e.g. the Caddy edge) which
    /// forwards cleartext to this gateway over a trusted interface. This is the
    /// production expectation.
    #[default]
    TerminatedUpstream,
}

impl TlsMode {
    /// A human-readable explanation of the seam for this mode, used in logs and
    /// operator diagnostics so the boundary is never silently assumed.
    pub fn reason(&self) -> &'static str {
        match self {
            Self::Disabled => {
                "TLS disabled: the gateway serves cleartext HTTP/1.1; do not expose it directly to untrusted networks"
            }
            Self::TerminatedUpstream => {
                "TLS terminated by an external edge (e.g. Caddy); the gateway receives cleartext over a trusted interface and never performs a handshake"
            }
        }
    }

    /// Whether client connections to the gateway are expected to be cleartext.
    ///
    /// Always true: with std-only networking the gateway never negotiates TLS.
    pub fn gateway_listens_cleartext(&self) -> bool {
        true
    }
}

impl fmt::Display for TlsMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled => write!(f, "disabled"),
            Self::TerminatedUpstream => write!(f, "terminated-upstream"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_expects_external_termination() {
        assert_eq!(TlsMode::default(), TlsMode::TerminatedUpstream);
    }

    #[test]
    fn every_mode_listens_cleartext() {
        assert!(TlsMode::Disabled.gateway_listens_cleartext());
        assert!(TlsMode::TerminatedUpstream.gateway_listens_cleartext());
    }

    #[test]
    fn reasons_are_distinct_and_nonempty() {
        assert_ne!(TlsMode::Disabled.reason(), TlsMode::TerminatedUpstream.reason());
        assert!(!TlsMode::Disabled.reason().is_empty());
    }
}
