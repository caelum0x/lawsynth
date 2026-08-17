//! Hop-by-hop header stripping and `X-Forwarded-*` construction.
//!
//! A conforming proxy must not forward connection-specific ("hop-by-hop")
//! headers, per RFC 7230 §6.1. This module removes them from both the request it
//! forwards upstream and the response it returns downstream, and it builds the
//! `X-Forwarded-*` set so the backend can recover the original client context.

/// Header names that describe a single transport hop and must never be relayed.
const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// True when a header is hop-by-hop and must be stripped before relaying.
pub fn is_hop_by_hop(name: &str) -> bool {
    HOP_BY_HOP.iter().any(|candidate| name.eq_ignore_ascii_case(candidate))
}

/// Returns a copy of `headers` with hop-by-hop and length headers removed.
///
/// `Content-Length` is dropped because the transport recomputes it from the
/// buffered body; forwarding a stale value would corrupt framing.
pub fn strip_hop_by_hop(headers: &[(String, String)]) -> Vec<(String, String)> {
    headers
        .iter()
        .filter(|(name, _)| !is_hop_by_hop(name) && !name.eq_ignore_ascii_case("content-length"))
        .cloned()
        .collect()
}

/// Appends `client_ip` to any existing `X-Forwarded-For` chain.
pub fn forwarded_for(existing: Option<&str>, client_ip: &str) -> String {
    match existing {
        Some(chain) if !chain.trim().is_empty() => format!("{chain}, {client_ip}"),
        _ => client_ip.to_owned(),
    }
}

/// Builds the forwarded request headers: original headers minus hop-by-hop,
/// plus the `X-Forwarded-*` trio describing the client and gateway hop.
pub fn build_forwarded_headers(
    original: &[(String, String)],
    client_ip: &str,
    proto: &str,
    host: Option<&str>,
) -> Vec<(String, String)> {
    let existing_xff = original
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("x-forwarded-for"))
        .map(|(_, value)| value.as_str());
    let xff = forwarded_for(existing_xff, client_ip);

    let mut headers: Vec<(String, String)> = original
        .iter()
        .filter(|(name, _)| {
            !is_hop_by_hop(name)
                && !name.eq_ignore_ascii_case("content-length")
                && !name.eq_ignore_ascii_case("x-forwarded-for")
                && !name.eq_ignore_ascii_case("x-forwarded-proto")
        })
        .cloned()
        .collect();

    headers.push(("X-Forwarded-For".into(), xff));
    headers.push(("X-Forwarded-Proto".into(), proto.to_owned()));
    if let Some(host) = host {
        // Only set X-Forwarded-Host when not already present from an outer proxy.
        if !original.iter().any(|(name, _)| name.eq_ignore_ascii_case("x-forwarded-host")) {
            headers.push(("X-Forwarded-Host".into(), host.to_owned()));
        }
    }
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_hop_by_hop_headers() {
        let input = vec![
            ("Connection".into(), "keep-alive".into()),
            ("Content-Type".into(), "application/json".into()),
            ("Transfer-Encoding".into(), "chunked".into()),
        ];
        let out = strip_hop_by_hop(&input);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].0, "Content-Type");
    }

    #[test]
    fn appends_to_existing_forwarded_chain() {
        assert_eq!(forwarded_for(Some("10.0.0.1"), "10.0.0.2"), "10.0.0.1, 10.0.0.2");
        assert_eq!(forwarded_for(None, "10.0.0.2"), "10.0.0.2");
        assert_eq!(forwarded_for(Some("  "), "10.0.0.2"), "10.0.0.2");
    }

    #[test]
    fn builds_forwarded_trio() {
        let original = vec![("Authorization".into(), "Bearer t".into())];
        let out = build_forwarded_headers(&original, "1.2.3.4", "https", Some("api.local"));
        assert!(out.iter().any(|(n, v)| n == "X-Forwarded-For" && v == "1.2.3.4"));
        assert!(out.iter().any(|(n, v)| n == "X-Forwarded-Proto" && v == "https"));
        assert!(out.iter().any(|(n, v)| n == "X-Forwarded-Host" && v == "api.local"));
        assert!(out.iter().any(|(n, v)| n == "Authorization" && v == "Bearer t"));
    }
}
