use lawsynth_regime::{SegmentationConfig, TransitionMatrix, pelt};
fn main() {
    let signal = [0.0, 0.1, -0.1, 4.9, 5.0, 5.1];
    let segmentation = pelt(&signal, SegmentationConfig { penalty: 0.5, min_segment_len: 2 })
        .expect("valid signal");
    let states: Vec<usize> = (0..signal.len()).map(|i| segmentation.label_at(i).unwrap()).collect();
    let transitions =
        TransitionMatrix::from_states(&states, segmentation.segments.len()).expect("labels");
    println!(
        "change points: {:?}; transitions: {:?}",
        segmentation.change_points(),
        transitions.counts
    );
}
