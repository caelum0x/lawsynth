//! Registry lookup: deterministic ordering, resolution, and unknown-name errors.

use lawsynth_domains::{DomainPresetKind, PresetError, all, names, preset};

#[test]
fn names_are_deterministic_and_stable() {
    assert_eq!(names(), vec!["damped-oscillator", "lotka-volterra", "brusselator"]);
    // Repeated calls return the same order.
    assert_eq!(names(), names());
}

#[test]
fn every_name_resolves_to_its_kind() {
    for kind in DomainPresetKind::ALL {
        let resolved = preset(kind.name()).expect("registered name resolves");
        assert_eq!(resolved.name(), kind.name());
        assert_eq!(resolved, kind.build());
    }
}

#[test]
fn unknown_name_is_a_reported_error() {
    let error = preset("schrodinger").unwrap_err();
    match &error {
        PresetError::Unknown { requested, available } => {
            assert_eq!(requested, "schrodinger");
            assert_eq!(available, &names());
        }
    }
    // The message is actionable: it names the miss and lists what is available.
    let message = error.to_string();
    assert!(message.contains("schrodinger"));
    assert!(message.contains("brusselator"));
}

#[test]
fn all_presets_are_built_in_canonical_order() {
    let built = all();
    assert_eq!(built.len(), DomainPresetKind::ALL.len());
    let built_names: Vec<&str> = built.iter().map(|preset| preset.name()).collect();
    assert_eq!(built_names, names());
}
