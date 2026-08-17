//! The admission pipeline and blocking accept loop.
//!
//! [`Gateway`] wires the individual policy modules into one ordered pipeline and
//! runs a thread-per-connection server over `std::net`. Time is supplied by an
//! injected [`Clock`] so the rate limiter — and therefore the whole pipeline —
//! is deterministic in tests. The pipeline order is deliberate: cheap local
//! endpoints first, then CORS preflight, rate limiting, the route allowlist,
//! edge authentication, the body-size ceiling, and only then a socket to the
//! backend.

use crate::GatewayError;
use crate::auth::{self, AuthDecision};
use crate::body_limits::{self, BodyCheck};
use crate::config::GatewayConfig;
use crate::cors;
use crate::health;
use crate::http::{self, HttpRequest, HttpResponse, ReadOutcome};
use crate::metrics::Metrics;
use crate::proxy;
use crate::rate_limit::{RateDecision, RateLimiter};
use crate::routing::{self, RouteDecision};
use crate::shutdown::Shutdown;
use crate::tracing::{RequestIds, request_log_line};
use std::io::{self, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Supplies the current Unix time in seconds to the deterministic pipeline.
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// Interval the accept loop waits between non-blocking accept polls.
const ACCEPT_POLL: Duration = Duration::from_millis(50);

/// A configured gateway: shared, cloneable per-connection state plus the policy
/// pipeline that turns a client request into a response.
#[derive(Clone)]
pub struct Gateway {
    config: Arc<GatewayConfig>,
    clock: Clock,
    metrics: Arc<Metrics>,
    limiter: Arc<RateLimiter>,
    request_ids: Arc<RequestIds>,
}

impl Gateway {
    /// Builds a gateway from a validated config and an explicit clock.
    pub fn new(config: GatewayConfig, clock: Clock) -> Result<Self, GatewayError> {
        config.validate()?;
        let limiter = RateLimiter::new(config.rate_limit_quota, config.rate_limit_window.as_secs());
        Ok(Self {
            config: Arc::new(config),
            clock,
            metrics: Arc::new(Metrics::new()),
            limiter: Arc::new(limiter),
            request_ids: Arc::new(RequestIds::new()),
        })
    }

    /// Builds a gateway whose clock reads the system wall clock in Unix seconds.
    pub fn with_system_clock(config: GatewayConfig) -> Result<Self, GatewayError> {
        let clock: Clock = Arc::new(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_secs())
                .unwrap_or(0)
        });
        Self::new(config, clock)
    }

    /// The immutable configuration backing this gateway.
    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }

    /// A snapshot of the current request counters.
    pub fn metrics_snapshot(&self) -> crate::metrics::MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Runs the full admission pipeline for a parsed request, records metrics,
    /// and stamps the response with its correlation identifier.
    pub fn handle(&self, request: &HttpRequest, client_ip: &str) -> HttpResponse {
        let request_id = self.request_ids.next_id();
        let response = self.dispatch(request, client_ip);
        self.metrics.record(response.status);
        response.with_header("X-Request-Id", request_id)
    }

    /// The ordered policy pipeline, absent metrics/identifier bookkeeping.
    fn dispatch(&self, request: &HttpRequest, client_ip: &str) -> HttpResponse {
        // Local endpoints are answered by the gateway itself and never proxied,
        // rate-limited, or authenticated.
        if request.path == "/healthz" {
            return health::healthz(&self.config);
        }
        if request.path == "/metrics" {
            let text = self.metrics.snapshot().render();
            return HttpResponse::bytes(200, "text/plain; version=0.0.4", text.into_bytes());
        }

        let origin = request.header("origin");

        // CORS preflight is resolved before any admission checks.
        if let Some(preflight) =
            cors::preflight(&request.method, origin, &self.config.allowed_origins)
        {
            return preflight;
        }

        // Rate limiting, keyed by client IP, using the injected clock.
        let now = (self.clock)();
        if let RateDecision::Limited { retry_after } = self.limiter.check(client_ip, now) {
            self.metrics.record_rate_limited();
            return HttpResponse::error_code(429, "rate_limited", "request quota exceeded")
                .with_header("Retry-After", retry_after.to_string());
        }

        // Route allowlist: reject unknown paths and methods before the backend.
        let protected =
            match routing::resolve(&request.method, &request.path, &self.config.api_prefix) {
                RouteDecision::Proxy { protected } => protected,
                RouteDecision::NotFound => {
                    return HttpResponse::error_code(
                        404,
                        "not_found",
                        "no route matches the request",
                    );
                }
                RouteDecision::MethodNotAllowed { allow } => {
                    return HttpResponse::error_code(
                        405,
                        "method_not_allowed",
                        "method not supported",
                    )
                    .with_header("Allow", allow.join(", "));
                }
            };

        // Edge authentication for protected routes.
        match auth::authorize(protected, request.header("authorization")) {
            AuthDecision::Authorized => {}
            AuthDecision::Missing | AuthDecision::Malformed => {
                return HttpResponse::error_code(401, "unauthorized", "a bearer token is required")
                    .with_header("WWW-Authenticate", "Bearer");
            }
        }

        // Body-size ceiling.
        if let BodyCheck::TooLarge { .. } =
            body_limits::check(request.body.len(), self.config.max_body_bytes)
        {
            return HttpResponse::error_code(413, "payload_too_large", "request body too large");
        }

        // Forward to the upstream and reflect CORS headers for allowed origins.
        let response = match proxy::forward(&self.config, request, client_ip) {
            Ok(response) => response,
            Err(error) => HttpResponse::error(&error),
        };
        cors::apply(response, origin, &self.config.allowed_origins)
    }

    /// Serves connections until `shutdown` is triggered, one thread each.
    pub fn serve(&self, listener: &TcpListener, shutdown: &Shutdown) -> io::Result<()> {
        listener.set_nonblocking(true)?;
        loop {
            if shutdown.is_triggered() {
                return Ok(());
            }
            match listener.accept() {
                Ok((stream, peer)) => {
                    stream.set_nonblocking(false)?;
                    let worker = self.clone();
                    let client_ip = peer.ip().to_string();
                    thread::spawn(move || {
                        if let Err(error) = worker.serve_connection(stream, &client_ip) {
                            eprintln!("lawsynth-gateway: connection error: {error}");
                        }
                    });
                }
                Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(ACCEPT_POLL);
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn serve_connection(&self, stream: TcpStream, client_ip: &str) -> io::Result<()> {
        let start = Instant::now();
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = stream;

        let outcome = http::read_request(
            &mut reader,
            self.config.max_body_bytes,
            self.config.max_header_bytes,
            self.config.max_headers,
        )?;

        let (response, method, path) = match outcome {
            ReadOutcome::Closed => return Ok(()),
            ReadOutcome::PayloadTooLarge => (
                self.reject(413, "payload_too_large", "request body too large"),
                "-".into(),
                "-".into(),
            ),
            ReadOutcome::HeaderFieldsTooLarge => (
                self.reject(431, "header_fields_too_large", "request headers too large"),
                "-".into(),
                "-".into(),
            ),
            ReadOutcome::Request(request) => {
                let response = self.handle(&request, client_ip);
                (response, request.method.clone(), request.path.clone())
            }
        };

        let request_id = response.header("x-request-id").unwrap_or("-").to_owned();
        let line = request_log_line(
            &request_id,
            &method,
            &path,
            response.status,
            client_ip,
            start.elapsed().as_micros(),
        );
        eprintln!("{line}");

        http::write_response(&mut writer, &response)?;
        writer.flush()
    }

    /// Builds a transport-level rejection response, records it, and stamps an id.
    ///
    /// Read-level rejections never reach [`Self::handle`], so metrics and the
    /// correlation identifier are applied here instead.
    fn reject(&self, status: u16, code: &str, message: &str) -> HttpResponse {
        self.metrics.record(status);
        let request_id = self.request_ids.next_id();
        HttpResponse::error_code(status, code, message).with_header("X-Request-Id", request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gateway_at(now: u64) -> Gateway {
        let clock: Clock = Arc::new(move || now);
        Gateway::new(GatewayConfig::new("127.0.0.1:0", "127.0.0.1:9"), clock).unwrap()
    }

    #[test]
    fn healthz_is_answered_locally() {
        let gateway = gateway_at(0);
        let request = HttpRequest::new("GET", "/healthz", Vec::new(), Vec::new());
        let response = gateway.handle(&request, "127.0.0.1");
        assert_eq!(response.status, 200);
        assert!(response.header("x-request-id").is_some());
    }

    #[test]
    fn unknown_route_is_404_without_touching_upstream() {
        let gateway = gateway_at(0);
        let request = HttpRequest::new("GET", "/v1/nope", Vec::new(), Vec::new());
        assert_eq!(gateway.handle(&request, "127.0.0.1").status, 404);
    }

    #[test]
    fn protected_route_without_token_is_401() {
        let gateway = gateway_at(0);
        let request = HttpRequest::new("POST", "/v1/runs", Vec::new(), Vec::new());
        assert_eq!(gateway.handle(&request, "127.0.0.1").status, 401);
    }
}
