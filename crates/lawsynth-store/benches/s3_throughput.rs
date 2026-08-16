use lawsynth_store::{MemoryStore, ObjectKey, ObjectStore};
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let store = MemoryStore::default();
    let start = Instant::now();
    let mut bytes = 0usize;
    for index in 0..10_000 {
        let payload = vec![index as u8; 256];
        bytes += payload.len();
        store
            .put(
                ObjectKey::new(format!("throughput/{index}")).unwrap(),
                payload,
            )
            .unwrap();
    }
    black_box(store.object_count());
    println!(
        "memory object writes: {} MiB/s",
        bytes as f64 / start.elapsed().as_secs_f64() / 1_048_576.0
    );
}
