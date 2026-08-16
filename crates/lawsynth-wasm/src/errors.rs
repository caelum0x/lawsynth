use crate::WasmError;
/// Stable machine-facing codes for errors at the embedding boundary.
pub fn code(error: &WasmError) -> &'static str {
    match error {
        WasmError::InvalidWorld(_) => "INVALID_WORLD",
        WasmError::InvalidExpression(_) => "INVALID_EXPRESSION",
        WasmError::InvalidBundle(_) => "INVALID_BUNDLE",
        WasmError::InvalidTrajectory(_) => "INVALID_TRAJECTORY",
        WasmError::Simulation(_) => "SIMULATION_FAILED",
        WasmError::MemoryLimit { .. } => "MEMORY_LIMIT",
        WasmError::Unsupported(_) => "UNSUPPORTED",
    }
}
