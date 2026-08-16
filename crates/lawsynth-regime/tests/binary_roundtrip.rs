use lawsynth_regime::{DiscreteHmm, TransitionMatrix, best_binary_split};
#[test]
fn split_and_state_transition_estimates_are_observed() {
    let data = [0.0, 0.0, 0.0, 4.0, 4.0, 4.0];
    assert_eq!(best_binary_split(&data, 2).unwrap().unwrap().index, 3);
    let matrix = TransitionMatrix::from_states(&[0, 1, 1, 0], 2).unwrap();
    assert_eq!(matrix.counts[0][1], 1);
    let hmm = DiscreteHmm {
        initial: vec![0.5, 0.5],
        transition: vec![vec![0.9, 0.1], vec![0.1, 0.9]],
        emission: vec![vec![0.9, 0.1], vec![0.1, 0.9]],
    };
    assert_eq!(hmm.viterbi(&[0, 0, 1, 1]).unwrap().states, vec![0, 0, 1, 1]);
}
