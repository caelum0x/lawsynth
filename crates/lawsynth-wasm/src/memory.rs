use crate::WasmError;
/// Explicit byte accounting for hosts that copy payloads into a WASM linear memory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryBudget {
    limit: usize,
    used: usize,
}
impl MemoryBudget {
    pub fn new(limit: usize) -> Result<Self, WasmError> {
        if limit == 0 {
            return Err(WasmError::MemoryLimit {
                requested: 1,
                available: 0,
            });
        }
        Ok(Self { limit, used: 0 })
    }
    pub fn reserve(&mut self, bytes: usize) -> Result<(), WasmError> {
        let available = self.limit.saturating_sub(self.used);
        if bytes > available {
            return Err(WasmError::MemoryLimit {
                requested: bytes,
                available,
            });
        }
        self.used += bytes;
        Ok(())
    }
    pub fn release(&mut self, bytes: usize) {
        self.used = self.used.saturating_sub(bytes);
    }
    pub fn used(&self) -> usize {
        self.used
    }
    pub fn available(&self) -> usize {
        self.limit - self.used
    }
}
