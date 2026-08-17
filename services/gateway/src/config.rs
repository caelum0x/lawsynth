//! Validated gateway configuration.
//!
//! Every runtime limit the admission layer enforces is declared here and checked
//! by [`GatewayConfig::validate`] before a listener is bound. Defaults mirror
//! `services/gateway/.env.example` so the Rust proxy and the legacy Python
//! prototype agree on the same operational envelope.

use crate::errors::GatewayError;
use crate::tls::TlsMode;
use std::time::Duration;

/// Absolute request-body ceiling (64 MiB), matching the artifact upload limit.
const DEFAULT_MAX_BODY_BYTES: usize = 64 * 1024 * 1024;
/// Request line + header block ceiling (32 KiB).
const DEFAULT_MAX_HEADER_BYTES: usize = 32 * 1024;
/// Maximum number of individual request headers accepted.
const DEFAULT_MAX_HEADERS: usize = 64;
/// Requests permitted per client key within one rate-limit window.
const DEFAULT_RATE_QUOTA: u32 = 120;
/// Rate-limit window length in seconds.
const DEFAULT_RATE_WINDOW_SECONDS: u64 = 60;
/// Default upstream connect/read timeout.
const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
/// Default versioned API prefix the gateway is willing to proxy.
const DEFAULT_API_PREFIX: &str = "/v1";

/// The complete, validated operating envelope of the gateway.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayConfig {
    /// Local address the gateway listens on, e.g. `127.0.0.1:8080`.
    pub listen_addr: String,
    /// Upstream backend address the gateway forwards to, e.g. `127.0.0.1:9000`.
    pub upstream_addr: String,
    /// Maximum accepted request body in bytes; larger bodies yield `413`.
    pub max_body_bytes: usize,
    /// Maximum accepted request line + header block in bytes.
    pub max_header_bytes: usize,
    /// Maximum number of request headers.
    pub max_headers: usize,
    /// Requests permitted per client key per window before `429`.
    pub rate_limit_quota: u32,
    /// Sliding/fixed window length for the rate limiter.
    pub rate_limit_window: Duration,
    /// Exact browser origins permitted by CORS; empty disables CORS.
    pub allowed_origins: Vec<String>,
    /// Versioned prefix under which upstream routes are exposed.
    pub api_prefix: String,
    /// Connect and read timeout applied to the upstream socket.
    pub request_timeout: Duration,
    /// How TLS is expected to be terminated for this deployment.
    pub tls_mode: TlsMode,
}

impl GatewayConfig {
    /// Builds a configuration with documented defaults for the given endpoints.
    pub fn new(listen_addr: impl Into<String>, upstream_addr: impl Into<String>) -> Self {
        Self {
            listen_addr: listen_addr.into(),
            upstream_addr: upstream_addr.into(),
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
            max_header_bytes: DEFAULT_MAX_HEADER_BYTES,
            max_headers: DEFAULT_MAX_HEADERS,
            rate_limit_quota: DEFAULT_RATE_QUOTA,
            rate_limit_window: Duration::from_secs(DEFAULT_RATE_WINDOW_SECONDS),
            allowed_origins: Vec::new(),
            api_prefix: DEFAULT_API_PREFIX.to_owned(),
            request_timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECONDS),
            tls_mode: TlsMode::default(),
        }
    }

    /// Replaces the allowed CORS origins, returning the updated config.
    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// Rejects any configuration that would produce undefined runtime behaviour.
    pub fn validate(&self) -> Result<(), GatewayError> {
        if self.listen_addr.trim().is_empty() {
            return Err(GatewayError::InvalidConfig("listen address must not be empty".into()));
        }
        if self.upstream_addr.trim().is_empty() {
            return Err(GatewayError::InvalidConfig("upstream address must not be empty".into()));
        }
        if self.max_body_bytes == 0 {
            return Err(GatewayError::InvalidConfig("max body bytes must be positive".into()));
        }
        if self.max_header_bytes < 256 {
            return Err(GatewayError::InvalidConfig(
                "max header bytes must leave room for a request line".into(),
            ));
        }
        if self.max_headers == 0 {
            return Err(GatewayError::InvalidConfig("max headers must be positive".into()));
        }
        if self.rate_limit_quota == 0 {
            return Err(GatewayError::InvalidConfig("rate-limit quota must be positive".into()));
        }
        if self.rate_limit_window.is_zero() {
            return Err(GatewayError::InvalidConfig("rate-limit window must be positive".into()));
        }
        if self.request_timeout.is_zero() {
            return Err(GatewayError::InvalidConfig("request timeout must be positive".into()));
        }
        if !self.api_prefix.starts_with('/') || self.api_prefix.len() < 2 {
            return Err(GatewayError::InvalidConfig(
                "api prefix must be an absolute path such as /v1".into(),
            ));
        }
        for origin in &self.allowed_origins {
            if !origin.starts_with("http://") && !origin.starts_with("https://") {
                return Err(GatewayError::InvalidConfig(format!(
                    "allowed origin must be an absolute URL: {origin}"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_validate() {
        assert!(GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000").validate().is_ok());
    }

    #[test]
    fn rejects_empty_upstream() {
        let config = GatewayConfig::new("127.0.0.1:8080", "");
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_bad_prefix() {
        let mut config = GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000");
        config.api_prefix = "v1".into();
        assert!(config.validate().is_err());
    }
}
