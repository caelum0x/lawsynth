use lawsynth_gateway::headers::{
    build_forwarded_headers, forwarded_for, is_hop_by_hop, strip_hop_by_hop,
};

#[test]
fn hop_by_hop_headers_are_recognised() {
    assert!(is_hop_by_hop("Connection"));
    assert!(is_hop_by_hop("transfer-encoding"));
    assert!(is_hop_by_hop("Upgrade"));
    assert!(!is_hop_by_hop("Content-Type"));
    assert!(!is_hop_by_hop("Authorization"));
}

#[test]
fn stripping_removes_hop_by_hop_and_length() {
    let input = vec![
        ("Connection".into(), "close".into()),
        ("Content-Length".into(), "5".into()),
        ("Authorization".into(), "Bearer t".into()),
    ];
    let out = strip_hop_by_hop(&input);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].0, "Authorization");
}

#[test]
fn forwarded_for_chains_client_addresses() {
    assert_eq!(forwarded_for(Some("203.0.113.1"), "198.51.100.2"), "203.0.113.1, 198.51.100.2");
    assert_eq!(forwarded_for(None, "198.51.100.2"), "198.51.100.2");
}

#[test]
fn forwarded_headers_add_the_x_forwarded_trio() {
    let original = vec![
        ("Host".into(), "gateway.local".into()),
        ("Connection".into(), "keep-alive".into()),
        ("Authorization".into(), "Bearer t".into()),
    ];
    let out = build_forwarded_headers(&original, "203.0.113.9", "https", Some("gateway.local"));

    assert!(!out.iter().any(|(name, _)| name.eq_ignore_ascii_case("connection")));
    assert!(out.iter().any(|(n, v)| n == "X-Forwarded-For" && v == "203.0.113.9"));
    assert!(out.iter().any(|(n, v)| n == "X-Forwarded-Proto" && v == "https"));
    assert!(out.iter().any(|(n, v)| n == "X-Forwarded-Host" && v == "gateway.local"));
    assert!(out.iter().any(|(n, v)| n == "Authorization" && v == "Bearer t"));
}

#[test]
fn existing_forwarded_for_is_extended_not_replaced() {
    let original = vec![("X-Forwarded-For".into(), "203.0.113.1".into())];
    let out = build_forwarded_headers(&original, "198.51.100.2", "http", None);
    let xff = out.iter().find(|(name, _)| name == "X-Forwarded-For").map(|(_, v)| v.as_str());
    assert_eq!(xff, Some("203.0.113.1, 198.51.100.2"));
}
