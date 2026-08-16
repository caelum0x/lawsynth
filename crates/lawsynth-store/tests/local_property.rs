use lawsynth_store::{LocalStore, ObjectKey, ObjectStore, StoreConfig};
use std::fs;
#[test]
fn local_store_roundtrips_nested_keys_without_escape() {
    let root = std::env::temp_dir().join(format!("lawsynth-store-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let store = LocalStore::open(&root, StoreConfig::default()).unwrap();
    let key = ObjectKey::new("a/b/result.bin").unwrap();
    store.put(key.clone(), vec![7, 8, 9]).unwrap();
    assert_eq!(store.get(&key).unwrap().bytes, vec![7, 8, 9]);
    assert_eq!(store.list(None).unwrap(), vec![key.clone()]);
    assert!(store.delete(&key).unwrap());
    assert!(store.list(None).unwrap().is_empty());
    fs::remove_dir_all(root).unwrap();
}
