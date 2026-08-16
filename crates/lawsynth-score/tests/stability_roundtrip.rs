use lawsynth_score::selection_stability;

#[test]
fn selection_stability_is_deterministic_and_reports_empty_agreement() {
    let selections = vec![vec![true, false, true], vec![true, true, false]];
    let first = selection_stability(&selections).unwrap();
    assert_eq!(first, selection_stability(&selections).unwrap());
    let empty = selection_stability(&[vec![false, false], vec![false, false]]).unwrap();
    assert_eq!(empty.mean_pairwise_jaccard, 1.0);
    assert_eq!(empty.selection_frequencies, vec![0.0, 0.0]);
}
