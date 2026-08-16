use crate::{Object, ObjectKey, ObjectStore, StoreError};
/// Connection settings validated for an S3-compatible endpoint.
/// This crate deliberately has no HTTP/TLS dependency. Remote calls return `Unsupported`;
/// applications must provide a signed HTTP transport adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
}
impl S3Config {
    pub fn validate(&self) -> Result<(), StoreError> {
        if !(self.endpoint.starts_with("https://") || self.endpoint.starts_with("http://")) {
            return Err(StoreError::Unsupported(
                "S3 endpoint must be an HTTP(S) URL".into(),
            ));
        }
        if self.bucket.is_empty() || self.bucket.contains('/') || self.region.is_empty() {
            return Err(StoreError::Unsupported(
                "S3 bucket and region must be nonempty".into(),
            ));
        }
        Ok(())
    }
    pub fn object_url(&self, key: &ObjectKey) -> Result<String, StoreError> {
        self.validate()?;
        Ok(format!(
            "{}/{}/{}",
            self.endpoint.trim_end_matches('/'),
            self.bucket,
            key.as_str()
        ))
    }
}
#[derive(Clone, Debug)]
pub struct S3Store {
    config: S3Config,
}
impl S3Store {
    pub fn new(config: S3Config) -> Result<Self, StoreError> {
        config.validate()?;
        Ok(Self { config })
    }
    pub fn config(&self) -> &S3Config {
        &self.config
    }
}
impl ObjectStore for S3Store {
    fn put(&self, _: ObjectKey, _: Vec<u8>) -> Result<Object, StoreError> {
        Err(StoreError::Unsupported(
            "S3 transport is not linked; supply an HTTP transport adapter".into(),
        ))
    }
    fn get(&self, _: &ObjectKey) -> Result<Object, StoreError> {
        Err(StoreError::Unsupported(
            "S3 transport is not linked; supply an HTTP transport adapter".into(),
        ))
    }
    fn delete(&self, _: &ObjectKey) -> Result<bool, StoreError> {
        Err(StoreError::Unsupported(
            "S3 transport is not linked; supply an HTTP transport adapter".into(),
        ))
    }
    fn list(&self, _: Option<&str>) -> Result<Vec<ObjectKey>, StoreError> {
        Err(StoreError::Unsupported(
            "S3 transport is not linked; supply an HTTP transport adapter".into(),
        ))
    }
}
