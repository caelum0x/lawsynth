mod support;

use lawsynth_gateway::HttpRequest;
use lawsynth_gateway::auth::{AuthDecision, authorize};

#[test]
fn public_routes_do_not_require_a_token() {
    assert_eq!(authorize(false, None), AuthDecision::Authorized);
}

#[test]
fn protected_route_missing_header_is_rejected() {
    assert_eq!(authorize(true, None), AuthDecision::Missing);
}

#[test]
fn bearer_scheme_is_case_insensitive() {
    assert_eq!(authorize(true, Some("Bearer token-value")), AuthDecision::Authorized);
    assert_eq!(authorize(true, Some("bEaReR token-value")), AuthDecision::Authorized);
}

#[test]
fn malformed_credentials_are_rejected() {
    assert_eq!(authorize(true, Some("Basic abc")), AuthDecision::Malformed);
    assert_eq!(authorize(true, Some("Bearer   ")), AuthDecision::Malformed);
}

#[test]
fn gateway_rejects_protected_route_without_token() {
    // No upstream is contacted; the 401 is produced at the edge.
    let gateway = support::gateway("127.0.0.1:9", 0);
    let request = HttpRequest::new("POST", "/v1/runs", Vec::new(), Vec::new());
    let response = gateway.handle(&request, "127.0.0.1");
    assert_eq!(response.status, 401);
    assert_eq!(response.header("www-authenticate"), Some("Bearer"));
}
