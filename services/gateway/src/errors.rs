//! Gateway error taxonomy.
//!
//! These variants distinguish operator-facing configuration faults from the
//! runtime failure modes of the upstream leg. Transport rejections that the
//! gateway itself produces (401/404/405/413/429) are modelled as
//! [`crate::http::HttpResponse`] values rather than errors, because they are a
//! normal, expected part of admission control; `GatewayError` is reserved for
//! conditions that abort request handling.

use std::fmt;

/// Failures the gateway can encounter while validating configuration or while
/// talking to the upstream backend.
#[derive(Debug)]
pub enum GatewayError {
    /// A `GatewayConfig` field failed validation at startup.
    InvalidConfig(String),
    /// The upstream connection could not be established or was reset.
    UpstreamUnavailable(String),
    /// The upstream did not respond within the configured read timeout.
    UpstreamTimeout,
    /// The upstream sent a response the gateway could not parse.
    BadUpstreamResponse(String),
    /// The forwarded body or upstream body exceeded a streaming ceiling.
    PayloadTooLarge,
}

impl GatewayError {
    /// The HTTP status the gateway returns to the client for this failure.
    pub fn status(&self) -> u16 {
        match self {
            Self::InvalidConfig(_) => 500,
            Self::UpstreamUnavailable(_) => 502,
            Self::UpstreamTimeout => 504,
            Self::BadUpstreamResponse(_) => 502,
            Self::PayloadTooLarge => 413,
        }
    }

    /// A stable machine-readable code for the error envelope.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidConfig(_) => "invalid_config",
            Self::UpstreamUnavailable(_) => "bad_gateway",
            Self::UpstreamTimeout => "gateway_timeout",
            Self::BadUpstreamResponse(_) => "bad_gateway",
            Self::PayloadTooLarge => "payload_too_large",
        }
    }
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => write!(f, "invalid gateway configuration: {reason}"),
            Self::UpstreamUnavailable(reason) => write!(f, "upstream unavailable: {reason}"),
            Self::UpstreamTimeout => write!(f, "upstream timed out"),
            Self::BadUpstreamResponse(reason) => write!(f, "malformed upstream response: {reason}"),
            Self::PayloadTooLarge => write!(f, "payload exceeds the configured maximum"),
        }
    }
}

impl std::error::Error for GatewayError {}
