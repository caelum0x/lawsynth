/// Limits applied by local and memory object stores.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreConfig {
    pub max_object_bytes: usize,
    pub cache_capacity_bytes: usize,
}
impl Default for StoreConfig {
    fn default() -> Self {
        Self { max_object_bytes: 128 * 1024 * 1024, cache_capacity_bytes: 32 * 1024 * 1024 }
    }
}
impl StoreConfig {
    pub fn validate(&self) -> Result<(), crate::StoreError> {
        if self.max_object_bytes == 0 {
            return Err(crate::StoreError::InvalidPart("max_object_bytes must be positive".into()));
        }
        Ok(())
    }
}
