//! Bearer-token admission at the edge.
//!
//! The gateway does not validate token *contents* — that is the backend's
//! responsibility, and the `Authorization` header is passed through untouched so
//! the upstream can authenticate it. What the edge enforces is *presence and
//! shape*: a protected route must carry a syntactically valid
//! `Authorization: Bearer <token>` header, so unauthenticated traffic is
//! rejected with `401` before it reaches the backend.

/// The result of checking a request's authorization for a protected route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthDecision {
    /// A well-formed bearer credential is present (or the route is public).
    Authorized,
    /// No `Authorization` header was supplied on a protected route.
    Missing,
    /// The header was present but not a well-formed `Bearer` credential.
    Malformed,
}

impl AuthDecision {
    pub fn is_authorized(&self) -> bool {
        matches!(self, Self::Authorized)
    }
}

/// Validates the presence and shape of a bearer credential.
///
/// Public routes (`protected == false`) always pass. Protected routes require a
/// non-empty token after a case-insensitive `Bearer ` scheme.
pub fn authorize(protected: bool, authorization: Option<&str>) -> AuthDecision {
    if !protected {
        return AuthDecision::Authorized;
    }
    let Some(value) = authorization else {
        return AuthDecision::Missing;
    };
    let trimmed = value.trim();
    let Some(rest) = strip_bearer(trimmed) else {
        return AuthDecision::Malformed;
    };
    if rest.trim().is_empty() { AuthDecision::Malformed } else { AuthDecision::Authorized }
}

/// Strips a case-insensitive `Bearer ` prefix, returning the token remainder.
fn strip_bearer(value: &str) -> Option<&str> {
    let scheme = "bearer ";
    if value.len() >= scheme.len() && value[..scheme.len()].eq_ignore_ascii_case(scheme) {
        Some(&value[scheme.len()..])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_routes_bypass_auth() {
        assert_eq!(authorize(false, None), AuthDecision::Authorized);
    }

    #[test]
    fn protected_route_requires_a_token() {
        assert_eq!(authorize(true, None), AuthDecision::Missing);
    }

    #[test]
    fn accepts_a_well_formed_bearer() {
        assert_eq!(authorize(true, Some("Bearer abc.def")), AuthDecision::Authorized);
        assert_eq!(authorize(true, Some("bearer abc.def")), AuthDecision::Authorized);
    }

    #[test]
    fn rejects_a_malformed_scheme() {
        assert_eq!(authorize(true, Some("Basic abc")), AuthDecision::Malformed);
        assert_eq!(authorize(true, Some("Bearer ")), AuthDecision::Malformed);
        assert_eq!(authorize(true, Some("token-only")), AuthDecision::Malformed);
    }
}
