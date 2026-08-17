//! Exact-origin CORS handling for browser clients.
//!
//! The policy is an *exact* allowlist: an `Origin` is reflected only if it
//! appears verbatim in the configured set. There is no wildcard and no prefix
//! matching, because credentialed cross-origin requests must not be served to an
//! unvetted origin. Preflight (`OPTIONS`) requests are answered locally; simple
//! and actual requests get the CORS response headers appended.

use crate::http::HttpResponse;

/// The CORS assessment of a request's `Origin`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OriginDecision {
    /// No `Origin` header: a non-browser or same-origin request. No CORS headers.
    NotCrossOrigin,
    /// The origin is on the allowlist and may be reflected.
    Allowed,
    /// The origin is present but not allowed; CORS headers are withheld.
    Denied,
}

/// Classifies an `Origin` header value against the configured allowlist.
pub fn classify(origin: Option<&str>, allowed: &[String]) -> OriginDecision {
    match origin {
        None => OriginDecision::NotCrossOrigin,
        Some(value) if allowed.iter().any(|candidate| candidate == value) => {
            OriginDecision::Allowed
        }
        Some(_) => OriginDecision::Denied,
    }
}

/// The CORS headers reflected for an allowed origin.
fn cors_headers(origin: &str) -> Vec<(String, String)> {
    vec![
        ("Access-Control-Allow-Origin".into(), origin.to_owned()),
        ("Access-Control-Allow-Credentials".into(), "true".into()),
        ("Access-Control-Allow-Methods".into(), "GET, POST, PUT, DELETE, OPTIONS".into()),
        ("Access-Control-Allow-Headers".into(), "Authorization, Content-Type".into()),
        ("Access-Control-Max-Age".into(), "600".into()),
        ("Vary".into(), "Origin".into()),
    ]
}

/// Builds a preflight response for an `OPTIONS` request, if CORS applies.
///
/// Returns `Some` only for a cross-origin preflight; an allowed origin gets a
/// `204` with the reflected headers, a denied origin a bare `403`.
pub fn preflight(method: &str, origin: Option<&str>, allowed: &[String]) -> Option<HttpResponse> {
    if !method.eq_ignore_ascii_case("OPTIONS") {
        return None;
    }
    match classify(origin, allowed) {
        OriginDecision::NotCrossOrigin => None,
        OriginDecision::Allowed => {
            let mut response = HttpResponse::empty(204);
            let origin = origin.expect("allowed implies an origin");
            for (name, value) in cors_headers(origin) {
                response = response.with_header(name, value);
            }
            Some(response)
        }
        OriginDecision::Denied => {
            Some(HttpResponse::error_code(403, "cors_denied", "origin is not permitted"))
        }
    }
}

/// Appends CORS response headers to a proxied response for an allowed origin.
pub fn apply(mut response: HttpResponse, origin: Option<&str>, allowed: &[String]) -> HttpResponse {
    if let OriginDecision::Allowed = classify(origin, allowed) {
        let origin = origin.expect("allowed implies an origin");
        for (name, value) in cors_headers(origin) {
            response = response.with_header(name, value);
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowlist() -> Vec<String> {
        vec!["https://app.lawsynth.dev".to_owned()]
    }

    #[test]
    fn exact_origin_is_allowed() {
        assert_eq!(
            classify(Some("https://app.lawsynth.dev"), &allowlist()),
            OriginDecision::Allowed
        );
    }

    #[test]
    fn other_origin_is_denied() {
        assert_eq!(classify(Some("https://evil.example"), &allowlist()), OriginDecision::Denied);
    }

    #[test]
    fn missing_origin_is_not_cross_origin() {
        assert_eq!(classify(None, &allowlist()), OriginDecision::NotCrossOrigin);
    }

    #[test]
    fn preflight_allowed_origin_returns_204() {
        let response =
            preflight("OPTIONS", Some("https://app.lawsynth.dev"), &allowlist()).unwrap();
        assert_eq!(response.status, 204);
        assert_eq!(
            response.header("access-control-allow-origin"),
            Some("https://app.lawsynth.dev")
        );
    }

    #[test]
    fn preflight_denied_origin_returns_403() {
        let response = preflight("OPTIONS", Some("https://evil.example"), &allowlist()).unwrap();
        assert_eq!(response.status, 403);
    }

    #[test]
    fn apply_reflects_allowed_origin_on_proxied_response() {
        let response =
            apply(HttpResponse::empty(200), Some("https://app.lawsynth.dev"), &allowlist());
        assert_eq!(
            response.header("access-control-allow-origin"),
            Some("https://app.lawsynth.dev")
        );
    }

    #[test]
    fn apply_is_a_noop_for_denied_origin() {
        let response = apply(HttpResponse::empty(200), Some("https://evil.example"), &allowlist());
        assert_eq!(response.header("access-control-allow-origin"), None);
    }
}
