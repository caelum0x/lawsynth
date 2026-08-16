use crate::{Object, ObjectKey, StoreError};
/// Synchronous object-store contract. Implementations are safe to share between threads.
pub trait ObjectStore: Send + Sync {
    fn put(&self, key: ObjectKey, bytes: Vec<u8>) -> Result<Object, StoreError>;
    fn get(&self, key: &ObjectKey) -> Result<Object, StoreError>;
    fn delete(&self, key: &ObjectKey) -> Result<bool, StoreError>;
    fn list(&self, prefix: Option<&str>) -> Result<Vec<ObjectKey>, StoreError>;
    fn contains(&self, key: &ObjectKey) -> Result<bool, StoreError> {
        match self.get(key) {
            Ok(_) => Ok(true),
            Err(StoreError::NotFound(_)) => Ok(false),
            Err(error) => Err(error),
        }
    }
}
