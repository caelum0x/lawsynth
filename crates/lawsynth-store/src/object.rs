use crate::StoreError;
use std::fmt;

/// A validated, portable object name. Keys are relative slash-separated paths.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct ObjectKey(String);
impl ObjectKey {
    pub fn new(key: impl Into<String>) -> Result<Self, StoreError> {
        let key = key.into();
        if key.is_empty()
            || key.starts_with('/')
            || key.ends_with('/')
            || key.contains('\\')
            || key.contains('\0')
            || key
                .split('/')
                .any(|s| s.is_empty() || s == "." || s == "..")
        {
            return Err(StoreError::InvalidKey(key));
        }
        Ok(Self(key))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for ObjectKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl TryFrom<&str> for ObjectKey {
    type Error = StoreError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Immutable object payload plus deterministic metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Object {
    pub bytes: Vec<u8>,
    pub checksum: u64,
}
impl Object {
    pub fn new(bytes: Vec<u8>) -> Self {
        let checksum = checksum(&bytes);
        Self { bytes, checksum }
    }
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    pub fn verify(&self) -> bool {
        checksum(&self.bytes) == self.checksum
    }
}
/// FNV-1a 64-bit checksum used for accidental-corruption detection, not security.
pub fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}
