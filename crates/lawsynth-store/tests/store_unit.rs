use lawsynth_store::{MemoryStore, ObjectKey, ObjectStore, StoreConfig, StoreError};
#[test]
fn validates_keys_limits_and_sorted_prefix_listing() {
    assert!(ObjectKey::new("../escape").is_err());
    let store = MemoryStore::new(StoreConfig {
        max_object_bytes: 3,
        cache_capacity_bytes: 8,
    })
    .unwrap();
    let key = ObjectKey::new("runs/a.bin").unwrap();
    assert!(matches!(
        store.put(key.clone(), vec![1, 2, 3, 4]),
        Err(StoreError::ObjectTooLarge { .. })
    ));
    let stored = store.put(key.clone(), vec![1, 2, 3]).unwrap();
    assert!(stored.verify());
    assert_eq!(store.get(&key).unwrap().bytes, vec![1, 2, 3]);
    store
        .put(ObjectKey::new("runs/b.bin").unwrap(), vec![])
        .unwrap();
    assert_eq!(store.list(Some("runs/")).unwrap().len(), 2);
    assert!(store.delete(&key).unwrap());
    assert!(!store.contains(&key).unwrap());
}
