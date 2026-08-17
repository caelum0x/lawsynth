use lawsynth_gateway::HttpResponse;
use lawsynth_gateway::cors::{OriginDecision, apply, classify, preflight};

fn allowlist() -> Vec<String> {
    vec!["https://app.lawsynth.dev".to_owned()]
}

#[test]
fn only_exact_origins_are_allowed() {
    assert_eq!(classify(Some("https://app.lawsynth.dev"), &allowlist()), OriginDecision::Allowed);
    assert_eq!(
        classify(Some("https://app.lawsynth.dev.evil"), &allowlist()),
        OriginDecision::Denied
    );
    assert_eq!(classify(None, &allowlist()), OriginDecision::NotCrossOrigin);
}

#[test]
fn preflight_for_allowed_origin_is_a_204_with_headers() {
    let response = preflight("OPTIONS", Some("https://app.lawsynth.dev"), &allowlist()).unwrap();
    assert_eq!(response.status, 204);
    assert_eq!(response.header("access-control-allow-origin"), Some("https://app.lawsynth.dev"));
    assert_eq!(response.header("access-control-allow-credentials"), Some("true"));
}

#[test]
fn preflight_for_denied_origin_is_forbidden() {
    let response = preflight("OPTIONS", Some("https://evil.example"), &allowlist()).unwrap();
    assert_eq!(response.status, 403);
}

#[test]
fn non_preflight_requests_are_not_intercepted() {
    assert!(preflight("GET", Some("https://app.lawsynth.dev"), &allowlist()).is_none());
}

#[test]
fn apply_reflects_only_allowed_origins() {
    let allowed = apply(HttpResponse::empty(200), Some("https://app.lawsynth.dev"), &allowlist());
    assert_eq!(allowed.header("access-control-allow-origin"), Some("https://app.lawsynth.dev"));

    let denied = apply(HttpResponse::empty(200), Some("https://evil.example"), &allowlist());
    assert_eq!(denied.header("access-control-allow-origin"), None);
}
