use lawsynth_plugin_host::validate_wasi_component;

#[test]
fn validates_wasm_header_before_runtime_handoff() {
    assert!(validate_wasi_component(b"\0asm\x01\0\0\0").is_ok());
    assert!(validate_wasi_component(b"not wasm").is_err());
}
