use crate::{Object, ObjectKey, ObjectStore, StoreError};
use std::collections::BTreeMap;
/// In-memory multipart assembly with strict, deterministic part ordering.
#[derive(Debug)]
pub struct MultipartUpload {
    key: ObjectKey,
    max_part_bytes: usize,
    parts: BTreeMap<u32, Vec<u8>>,
    closed: bool,
}
impl MultipartUpload {
    pub fn new(key: ObjectKey, max_part_bytes: usize) -> Result<Self, StoreError> {
        if max_part_bytes == 0 {
            return Err(StoreError::InvalidPart(
                "max_part_bytes must be positive".into(),
            ));
        }
        Ok(Self {
            key,
            max_part_bytes,
            parts: BTreeMap::new(),
            closed: false,
        })
    }
    pub fn add_part(&mut self, number: u32, bytes: Vec<u8>) -> Result<(), StoreError> {
        if self.closed {
            return Err(StoreError::InvalidPart(
                "upload is already finalized".into(),
            ));
        }
        if number == 0 || bytes.len() > self.max_part_bytes {
            return Err(StoreError::InvalidPart(
                "part number must be nonzero and within configured size".into(),
            ));
        }
        if self.parts.insert(number, bytes).is_some() {
            return Err(StoreError::InvalidPart(format!("duplicate part {number}")));
        }
        Ok(())
    }
    pub fn complete<S: ObjectStore>(&mut self, store: &S) -> Result<Object, StoreError> {
        if self.closed || self.parts.is_empty() {
            return Err(StoreError::InvalidPart(
                "upload has no completable parts".into(),
            ));
        }
        let mut expected = 1_u32;
        let mut all = Vec::new();
        for (number, bytes) in &self.parts {
            if *number != expected {
                return Err(StoreError::InvalidPart(format!("missing part {expected}")));
            }
            expected += 1;
            all.extend_from_slice(bytes);
        }
        let object = store.put(self.key.clone(), all)?;
        self.closed = true;
        Ok(object)
    }
    pub fn abort(&mut self) {
        self.parts.clear();
        self.closed = true;
    }
}
