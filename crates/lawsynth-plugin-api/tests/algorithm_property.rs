use lawsynth_plugin_api::{Frame, FrameKind, LifecycleEvent, PluginState};

#[test]
fn frame_roundtrip_is_lossless_and_state_machine_rejects_invalid_transition() {
    for size in [0, 1, 19, 1024] {
        let frame = Frame::new(FrameKind::Request, 7, vec![42; size]).unwrap();
        assert_eq!(Frame::decode(&frame.encode().unwrap()).unwrap(), frame);
    }
    assert!(PluginState::Discovered.transition(LifecycleEvent::Ready).is_err());
}
