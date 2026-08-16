use lawsynth_store::{MemoryStore, ObjectCache, ObjectKey, ObjectStore};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = MemoryStore::default();
    let key = ObjectKey::new("bundles/example.lsb")?;
    let object = store.put(key.clone(), b"validated bundle bytes".to_vec())?;
    let mut cache = ObjectCache::new(1024);
    cache.insert(key.clone(), object);
    println!(
        "{} bytes cached",
        cache.get(&key).expect("inserted object").len()
    );
    Ok(())
}
