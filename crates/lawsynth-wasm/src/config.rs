#[derive(Clone, Debug, PartialEq)]
pub struct WasmConfig {
    pub max_memory_bytes: usize,
    pub max_steps: usize,
    pub absolute_tolerance: f64,
}
impl Default for WasmConfig {
    fn default() -> Self {
        Self { max_memory_bytes: 64 * 1024 * 1024, max_steps: 1_000_000, absolute_tolerance: 1e-12 }
    }
}
impl WasmConfig {
    pub fn validate(&self) -> Result<(), crate::WasmError> {
        if self.max_memory_bytes == 0
            || self.max_steps == 0
            || !self.absolute_tolerance.is_finite()
            || self.absolute_tolerance <= 0.0
        {
            return Err(crate::WasmError::InvalidWorld(
                "invalid resource or tolerance limits".into(),
            ));
        }
        Ok(())
    }
}
