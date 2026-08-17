use crate::HostError;

/// Validate the WebAssembly binary header before passing a component to an
/// external WASI runtime. Execution is intentionally delegated: this crate
/// owns policy and framing, not a second WASI engine.
pub fn validate_wasi_component(bytes: &[u8]) -> Result<(), HostError> {
    if bytes.len() < 8 {
        return Err(HostError::Process("WASI component is truncated".into()));
    }
    if &bytes[..4] != b"\0asm" {
        return Err(HostError::Process("WASI component has invalid magic".into()));
    }
    if bytes[4..8] != [1, 0, 0, 0] {
        return Err(HostError::Process("unsupported WebAssembly version".into()));
    }
    Ok(())
}
