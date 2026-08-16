use lawsynth_regime::DiscreteHmm;
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let hmm = DiscreteHmm {
        initial: vec![0.5, 0.5],
        transition: vec![vec![0.95, 0.05], vec![0.05, 0.95]],
        emission: vec![vec![0.9, 0.1], vec![0.1, 0.9]],
    };
    let observations: Vec<usize> = (0..10_000).map(|i| usize::from(i % 40 >= 20)).collect();
    let start = Instant::now();
    black_box(hmm.viterbi(&observations).unwrap());
    println!("10k Viterbi sequence in {:?}", start.elapsed());
}
