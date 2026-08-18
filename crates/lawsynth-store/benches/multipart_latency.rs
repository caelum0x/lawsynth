use lawsynth_store::{MemoryStore, MultipartUpload, ObjectKey};
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let store = MemoryStore::default();
    let start = Instant::now();
    for index in 0..1_000 {
        let mut upload =
            MultipartUpload::new(ObjectKey::new(format!("multi/{index}")).unwrap(), 1024).unwrap();
        upload.add_part(1, vec![1; 512]).unwrap();
        upload.add_part(2, vec![2; 512]).unwrap();
        black_box(upload.complete(&store).unwrap());
    }
    println!("multipart completion: {:.2} us/op", start.elapsed().as_secs_f64() * 1e6 / 1000.0);
}
