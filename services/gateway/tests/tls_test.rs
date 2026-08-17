use lawsynth_gateway::TlsMode;

#[test]
fn default_mode_is_external_termination() {
    assert_eq!(TlsMode::default(), TlsMode::TerminatedUpstream);
}

#[test]
fn the_gateway_never_terminates_tls_itself() {
    // With std-only networking, both modes accept cleartext at the gateway; TLS
    // is always handled by an external terminator, never faked here.
    assert!(TlsMode::Disabled.gateway_listens_cleartext());
    assert!(TlsMode::TerminatedUpstream.gateway_listens_cleartext());
}

#[test]
fn each_mode_documents_a_distinct_boundary() {
    assert_ne!(TlsMode::Disabled.reason(), TlsMode::TerminatedUpstream.reason());
    assert!(TlsMode::TerminatedUpstream.reason().contains("external"));
    assert!(!TlsMode::Disabled.reason().is_empty());
}

#[test]
fn mode_renders_a_stable_label() {
    assert_eq!(TlsMode::Disabled.to_string(), "disabled");
    assert_eq!(TlsMode::TerminatedUpstream.to_string(), "terminated-upstream");
}
