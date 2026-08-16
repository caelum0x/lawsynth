use lawsynth_discovery::DiscoveryStage;

#[test]
fn stages_are_unique_totally_ordered_and_begin_with_validation() {
    let stages = DiscoveryStage::all();
    assert_eq!(stages[0], DiscoveryStage::Validate);
    assert_eq!(stages[stages.len() - 1], DiscoveryStage::Finalize);
    assert!(stages.windows(2).all(|pair| pair[0] < pair[1]));
}
