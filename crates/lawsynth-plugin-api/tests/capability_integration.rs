use lawsynth_plugin_api::{Capability, CapabilitySet};

#[test]
fn capability_sets_are_deterministic_and_subset_checked() {
    let declared = CapabilitySet::new([Capability::Algorithm, Capability::ReadDataset]);
    let granted = CapabilitySet::new([
        Capability::Algorithm,
        Capability::ReadDataset,
        Capability::WriteArtifact,
    ]);
    assert!(declared.is_subset_of(&granted));
    assert_eq!(
        declared.iter().map(Capability::as_str).collect::<Vec<_>>(),
        vec!["dataset.read", "algorithm"]
    );
}
