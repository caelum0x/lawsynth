//! Edge route allowlist for the versioned API surface.
//!
//! The gateway refuses to relay anything it does not explicitly recognise: an
//! unknown path is a `404` and a known path with an unsupported method is a
//! `405`, both decided *before* a socket to the backend is ever opened. This
//! keeps the upstream from having to defend against arbitrary traffic and gives
//! the edge a single, auditable list of what may cross it. Routing is a pure
//! function of `(method, path, prefix)` and holds no state.

/// The outcome of matching a request against the allowlist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteDecision {
    /// The route is permitted; forward it. `protected` requires a bearer token.
    Proxy { protected: bool },
    /// No route matches the path at all.
    NotFound,
    /// The path is known but the method is not; `allow` lists valid methods.
    MethodNotAllowed { allow: Vec<String> },
}

/// A single allowlisted route: a segment pattern plus its permitted methods.
struct Route {
    /// Path segments after the api prefix; `*` matches exactly one segment.
    segments: &'static [&'static str],
    methods: &'static [&'static str],
    protected: bool,
}

/// The static allowlist of routes the gateway will proxy under the api prefix.
const ROUTES: &[Route] = &[
    Route { segments: &["health"], methods: &["GET"], protected: false },
    Route { segments: &["runs"], methods: &["GET", "POST"], protected: true },
    Route { segments: &["runs", "*"], methods: &["GET", "DELETE"], protected: true },
    Route { segments: &["runs", "*", "status"], methods: &["GET"], protected: true },
    Route { segments: &["runs", "*", "cancel"], methods: &["POST"], protected: true },
    Route { segments: &["artifacts"], methods: &["POST"], protected: true },
    Route { segments: &["artifacts", "*"], methods: &["GET"], protected: true },
    Route { segments: &["datasets"], methods: &["GET", "POST"], protected: true },
    Route { segments: &["datasets", "*"], methods: &["GET"], protected: true },
];

/// Resolves `(method, path)` against the allowlist under `api_prefix`.
pub fn resolve(method: &str, path: &str, api_prefix: &str) -> RouteDecision {
    let Some(rest) = strip_prefix_segments(path, api_prefix) else {
        return RouteDecision::NotFound;
    };
    let request_segments: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();

    let mut path_matched = false;
    let mut allowed: Vec<String> = Vec::new();
    for route in ROUTES {
        if segments_match(route.segments, &request_segments) {
            path_matched = true;
            if route.methods.iter().any(|m| m.eq_ignore_ascii_case(method)) {
                return RouteDecision::Proxy { protected: route.protected };
            }
            for candidate in route.methods {
                let owned = (*candidate).to_owned();
                if !allowed.contains(&owned) {
                    allowed.push(owned);
                }
            }
        }
    }

    if path_matched {
        RouteDecision::MethodNotAllowed { allow: allowed }
    } else {
        RouteDecision::NotFound
    }
}

/// Returns the path remainder after `api_prefix`, or `None` if not under it.
fn strip_prefix_segments<'a>(path: &'a str, api_prefix: &str) -> Option<&'a str> {
    let prefix = api_prefix.trim_end_matches('/');
    if path == prefix {
        return Some("");
    }
    let with_slash = format!("{prefix}/");
    path.strip_prefix(&with_slash).map(|_| &path[with_slash.len() - 1..])
}

/// Whether a route pattern matches the request segments, honouring `*`.
fn segments_match(pattern: &[&str], request: &[&str]) -> bool {
    if pattern.len() != request.len() {
        return false;
    }
    pattern.iter().zip(request).all(|(p, r)| *p == "*" || p == r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxies_known_public_route() {
        assert_eq!(resolve("GET", "/v1/health", "/v1"), RouteDecision::Proxy { protected: false });
    }

    #[test]
    fn proxies_protected_collection_and_item() {
        assert_eq!(resolve("POST", "/v1/runs", "/v1"), RouteDecision::Proxy { protected: true });
        assert_eq!(
            resolve("GET", "/v1/runs/abc123", "/v1"),
            RouteDecision::Proxy { protected: true }
        );
        assert_eq!(
            resolve("GET", "/v1/runs/abc123/status", "/v1"),
            RouteDecision::Proxy { protected: true }
        );
    }

    #[test]
    fn unknown_path_is_not_found() {
        assert_eq!(resolve("GET", "/v1/unknown", "/v1"), RouteDecision::NotFound);
        assert_eq!(resolve("GET", "/other", "/v1"), RouteDecision::NotFound);
    }

    #[test]
    fn known_path_wrong_method_is_405_with_allow() {
        match resolve("DELETE", "/v1/runs", "/v1") {
            RouteDecision::MethodNotAllowed { allow } => {
                assert!(allow.contains(&"GET".to_owned()));
                assert!(allow.contains(&"POST".to_owned()));
            }
            other => panic!("expected 405, got {other:?}"),
        }
    }

    #[test]
    fn prefix_root_without_route_is_not_found() {
        assert_eq!(resolve("GET", "/v1", "/v1"), RouteDecision::NotFound);
    }
}
