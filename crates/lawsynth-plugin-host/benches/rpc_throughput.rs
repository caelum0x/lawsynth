use lawsynth_plugin_api::{Frame, FrameKind};
use lawsynth_plugin_host::{read_frame, write_frame};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let frame = Frame::new(FrameKind::Response, 12, vec![1; 512]).unwrap();
    let start = Instant::now();
    for _ in 0..100_000 {
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &frame).unwrap();
        black_box(read_frame(&mut bytes.as_slice()).unwrap());
    }
    println!("100000 host framed roundtrips in {:?}", start.elapsed());
}
