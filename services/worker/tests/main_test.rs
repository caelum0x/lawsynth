use lawsynth_worker::TransportSurface;

#[test]
fn only_the_in_process_typed_surface_is_advertised_as_available() {
    assert!(TransportSurface::LocalDirect.is_available());
    for surface in [TransportSurface::QueueNotImplemented, TransportSurface::NetworkNotImplemented]
    {
        assert!(!surface.is_available());
        assert!(!surface.reason().is_empty());
    }
}
