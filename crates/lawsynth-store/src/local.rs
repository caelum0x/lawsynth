use crate::{Object, ObjectKey, ObjectStore, StoreConfig, StoreError};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
/// File-backed object store. A key is always resolved under the configured root.
#[derive(Clone, Debug)]
pub struct LocalStore {
    root: Arc<PathBuf>,
    config: StoreConfig,
    lock: Arc<RwLock<()>>,
}
impl LocalStore {
    pub fn open(root: impl Into<PathBuf>, config: StoreConfig) -> Result<Self, StoreError> {
        config.validate()?;
        let root = root.into();
        fs::create_dir_all(&root)?;
        let root = fs::canonicalize(root)?;
        Ok(Self { root: Arc::new(root), config, lock: Arc::new(RwLock::new(())) })
    }
    pub fn root(&self) -> &Path {
        self.root.as_path()
    }
    fn path(&self, key: &ObjectKey) -> PathBuf {
        self.root.join(key.as_str())
    }
    fn keys_under(&self, directory: &Path, keys: &mut Vec<ObjectKey>) -> Result<(), StoreError> {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.keys_under(&path, keys)?;
            } else if path.is_file() {
                let relative = path
                    .strip_prefix(self.root.as_path())
                    .map_err(|_| StoreError::InvalidKey(path.display().to_string()))?;
                keys.push(ObjectKey::new(
                    relative.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"),
                )?);
            }
        }
        Ok(())
    }
}
impl ObjectStore for LocalStore {
    fn put(&self, key: ObjectKey, bytes: Vec<u8>) -> Result<Object, StoreError> {
        if bytes.len() > self.config.max_object_bytes {
            return Err(StoreError::ObjectTooLarge {
                actual: bytes.len(),
                limit: self.config.max_object_bytes,
            });
        }
        let _guard = self.lock.write().expect("local-store lock poisoned");
        let path = self.path(&key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("lawsynth-tmp");
        fs::write(&temporary, &bytes)?;
        fs::rename(temporary, path)?;
        Ok(Object::new(bytes))
    }
    fn get(&self, key: &ObjectKey) -> Result<Object, StoreError> {
        let _guard = self.lock.read().expect("local-store lock poisoned");
        let path = self.path(key);
        match fs::read(path) {
            Ok(bytes) => Ok(Object::new(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(StoreError::NotFound(key.to_string()))
            }
            Err(error) => Err(error.into()),
        }
    }
    fn delete(&self, key: &ObjectKey) -> Result<bool, StoreError> {
        let _guard = self.lock.write().expect("local-store lock poisoned");
        let path = self.path(key);
        match fs::remove_file(&path) {
            Ok(()) => {
                let mut parent = path.parent();
                while let Some(dir) = parent {
                    if dir == self.root.as_path() || fs::read_dir(dir)?.next().is_some() {
                        break;
                    }
                    fs::remove_dir(dir)?;
                    parent = dir.parent();
                }
                Ok(true)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }
    fn list(&self, prefix: Option<&str>) -> Result<Vec<ObjectKey>, StoreError> {
        let _guard = self.lock.read().expect("local-store lock poisoned");
        let mut keys = Vec::new();
        self.keys_under(self.root.as_path(), &mut keys)?;
        keys.sort();
        keys.retain(|key| prefix.is_none_or(|p| key.as_str().starts_with(p)));
        Ok(keys)
    }
}
