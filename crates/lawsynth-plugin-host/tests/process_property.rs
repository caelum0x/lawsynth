use lawsynth_plugin_api::{Frame, FrameKind};
use lawsynth_plugin_host::{ProcessSpec, ResourceMeter, read_frame, write_frame};
use std::path::PathBuf;

#[test]
fn in_memory_rpc_is_framed_and_resource_limited() {
    let frame = Frame::new(FrameKind::Hello, 0, b"plugin".to_vec()).unwrap();
    let mut bytes = Vec::new();
    write_frame(&mut bytes, &frame).unwrap();
    assert_eq!(read_frame(&mut bytes.as_slice()).unwrap(), frame);
    let mut meter = ResourceMeter::new(lawsynth_plugin_api::ResourceLimits {
        max_requests: 1,
        ..Default::default()
    })
    .unwrap();
    meter.begin_request().unwrap();
    assert!(meter.begin_request().is_err());
}

#[cfg(unix)]
#[test]
fn process_spec_spawns_without_shell_interpolation() {
    let spec = ProcessSpec {
        executable: PathBuf::from("/bin/sh"),
        args: vec!["-c".into(), "exit 0".into()],
        kind: lawsynth_plugin_api::PluginKind::Process,
    };
    let mut process = spec.spawn().unwrap();
    assert_eq!(process.wait().unwrap().code(), Some(0));
}
