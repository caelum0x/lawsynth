use lawsynth_plugin_api::{Frame, FrameKind};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let frame = Frame::new(FrameKind::Request, 1, vec![7; 256]).unwrap();
    let start = Instant::now();
    for _ in 0..100_000 {
        black_box(Frame::decode(&black_box(frame.encode().unwrap())).unwrap());
    }
    println!("100000 frame roundtrips in {:?}", start.elapsed());
}
