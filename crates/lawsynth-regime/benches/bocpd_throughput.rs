use lawsynth_regime::bocpd;
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let signal: Vec<f64> = (0..500).map(|i| if i < 250 { 0.0 } else { 4.0 }).collect();
    let start = Instant::now();
    black_box(bocpd(&signal, Default::default()).unwrap());
    println!("500 online updates in {:?}", start.elapsed());
}
