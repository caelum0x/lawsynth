use lawsynth_artifact_service::NetworkSurface;

#[test]
fn network_surface_does_not_claim_an_unimplemented_http_server() {
    assert!(!NetworkSurface::NotImplemented.supports_http());
    assert!(NetworkSurface::NotImplemented.reason().contains("no HTTP transport"));
}
