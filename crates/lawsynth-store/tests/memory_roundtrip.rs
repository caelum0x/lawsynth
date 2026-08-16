use lawsynth_store::{MemoryStore, ObjectKey, ObjectStore};
#[test]
fn replacement_is_atomic_from_reader_perspective() {
    let store = MemoryStore::default();
    let key = ObjectKey::new("artifact.bin").unwrap();
    for length in 0..128 {
        let bytes: Vec<u8> = (0..length).map(|v| v as u8).collect();
        let stored = store.put(key.clone(), bytes.clone()).unwrap();
        assert_eq!(stored.bytes, bytes);
        assert!(store.get(&key).unwrap().verify());
    }
}
