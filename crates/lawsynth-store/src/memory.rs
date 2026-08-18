use crate::{Object, ObjectKey, ObjectStore, StoreConfig, StoreError};
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
/// Thread-safe in-process object store suitable for tests and ephemeral workers.
#[derive(Clone, Debug)]
pub struct MemoryStore {
    config: StoreConfig,
    objects: Arc<RwLock<BTreeMap<ObjectKey, Object>>>,
}
impl MemoryStore {
    pub fn new(config: StoreConfig) -> Result<Self, StoreError> {
        config.validate()?;
        Ok(Self { config, objects: Arc::new(RwLock::new(BTreeMap::new())) })
    }
    pub fn object_count(&self) -> usize {
        self.objects.read().expect("memory-store lock poisoned").len()
    }
}
impl Default for MemoryStore {
    fn default() -> Self {
        Self::new(StoreConfig::default()).expect("default store config is valid")
    }
}
impl ObjectStore for MemoryStore {
    fn put(&self, key: ObjectKey, bytes: Vec<u8>) -> Result<Object, StoreError> {
        if bytes.len() > self.config.max_object_bytes {
            return Err(StoreError::ObjectTooLarge {
                actual: bytes.len(),
                limit: self.config.max_object_bytes,
            });
        }
        let object = Object::new(bytes);
        self.objects.write().expect("memory-store lock poisoned").insert(key, object.clone());
        Ok(object)
    }
    fn get(&self, key: &ObjectKey) -> Result<Object, StoreError> {
        self.objects
            .read()
            .expect("memory-store lock poisoned")
            .get(key)
            .cloned()
            .ok_or_else(|| StoreError::NotFound(key.to_string()))
    }
    fn delete(&self, key: &ObjectKey) -> Result<bool, StoreError> {
        Ok(self.objects.write().expect("memory-store lock poisoned").remove(key).is_some())
    }
    fn list(&self, prefix: Option<&str>) -> Result<Vec<ObjectKey>, StoreError> {
        Ok(self
            .objects
            .read()
            .expect("memory-store lock poisoned")
            .keys()
            .filter(|key| prefix.is_none_or(|p| key.as_str().starts_with(p)))
            .cloned()
            .collect())
    }
}
