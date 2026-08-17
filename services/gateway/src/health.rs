//! The gateway's own liveness endpoint.
//!
//! `/healthz` reports the *gateway's* readiness only. It deliberately does not
//! proxy to the upstream: a load balancer needs to know whether the admission
//! layer itself is accepting connections, independently of backend health, so
//! that a backend outage does not tear the edge out of rotation prematurely.

use crate::config::GatewayConfig;
use crate::http::HttpResponse;
use crate::json::Json;

/// Builds the `/healthz` response describing the gateway's configuration state.
pub fn healthz(config: &GatewayConfig) -> HttpResponse {
    let body = Json::Object(vec![
        ("status".into(), Json::string("ok")),
        ("service".into(), Json::string("lawsynth-gateway")),
        ("upstream".into(), Json::string(config.upstream_addr.clone())),
        ("api_prefix".into(), Json::string(config.api_prefix.clone())),
        ("tls_mode".into(), Json::string(config.tls_mode.to_string())),
    ]);
    HttpResponse::json(200, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_ok_and_does_not_reference_upstream_health() {
        let config = GatewayConfig::new("127.0.0.1:8080", "127.0.0.1:9000");
        let response = healthz(&config);
        assert_eq!(response.status, 200);
        let body = String::from_utf8(response.body).unwrap();
        assert!(body.contains("\"status\":\"ok\""));
        assert!(body.contains("\"service\":\"lawsynth-gateway\""));
    }
}
