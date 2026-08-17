//! LawSynth gateway: a std-only, dependency-free HTTP/1.1 reverse proxy.
//!
//! The gateway is the admission layer in front of the LawSynth API. It links no
//! async runtime, HTTP framework, or TLS crate — only `std` — and forwards
//! traffic to an upstream backend over a plain `std::net::TcpStream`. Every
//! policy is a small, independently testable module; [`Gateway`] composes them
//! into one ordered pipeline (see [`server`]).
//!
//! TLS is handled honestly: the standard library cannot terminate TLS, so the
//! gateway declares the termination seam in [`tls`] and expects an external edge
//! (the compose Caddy service) to terminate TLS and forward cleartext.

pub mod auth;
pub mod body_limits;
pub mod config;
pub mod cors;
pub mod downloads;
pub mod errors;
pub mod events;
pub mod headers;
pub mod health;
pub mod http;
pub mod json;
pub mod metrics;
pub mod proxy;
pub mod rate_limit;
pub mod retry;
pub mod routing;
pub mod server;
pub mod shutdown;
pub mod timeouts;
pub mod tls;
pub mod tracing;
pub mod uploads;

pub use config::GatewayConfig;
pub use errors::GatewayError;
pub use events::{EventLog, RequestEvent};
pub use http::{HttpRequest, HttpResponse};
pub use json::Json;
pub use metrics::{Metrics, MetricsSnapshot};
pub use rate_limit::{RateDecision, RateLimiter};
pub use routing::{RouteDecision, resolve as resolve_route};
pub use server::{Clock, Gateway};
pub use shutdown::{Shutdown, ShutdownHandle};
pub use tls::TlsMode;
