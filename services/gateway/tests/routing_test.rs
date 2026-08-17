use lawsynth_gateway::{RouteDecision, resolve_route};

#[test]
fn public_health_route_needs_no_token() {
    assert_eq!(
        resolve_route("GET", "/v1/health", "/v1"),
        RouteDecision::Proxy { protected: false }
    );
}

#[test]
fn protected_routes_are_marked_protected() {
    assert_eq!(
        resolve_route("POST", "/v1/artifacts", "/v1"),
        RouteDecision::Proxy { protected: true }
    );
    assert_eq!(
        resolve_route("GET", "/v1/artifacts/deadbeef", "/v1"),
        RouteDecision::Proxy { protected: true }
    );
}

#[test]
fn unknown_paths_are_not_found() {
    assert_eq!(resolve_route("GET", "/v1/does-not-exist", "/v1"), RouteDecision::NotFound);
    assert_eq!(resolve_route("GET", "/healthz", "/v1"), RouteDecision::NotFound);
}

#[test]
fn known_path_unknown_method_reports_allow() {
    match resolve_route("PATCH", "/v1/runs/abc", "/v1") {
        RouteDecision::MethodNotAllowed { allow } => {
            assert!(allow.contains(&"GET".to_owned()));
            assert!(allow.contains(&"DELETE".to_owned()));
        }
        other => panic!("expected method-not-allowed, got {other:?}"),
    }
}

#[test]
fn routing_honours_a_custom_prefix() {
    assert_eq!(
        resolve_route("GET", "/api/health", "/api"),
        RouteDecision::Proxy { protected: false }
    );
    assert_eq!(resolve_route("GET", "/v1/health", "/api"), RouteDecision::NotFound);
}
