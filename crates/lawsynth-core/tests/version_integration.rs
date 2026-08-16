use std::str::FromStr;

use lawsynth_core::{CURRENT_ENGINE_VERSION, EngineVersion};

#[test]
fn semantic_versions_round_trip_and_compare_by_major_contract() {
    let parsed = EngineVersion::from_str("2.14.7").unwrap();
    assert_eq!(parsed.to_string(), "2.14.7");
    assert!(parsed.is_compatible_with(EngineVersion::new(2, 0, 0)));
    assert!(!parsed.is_compatible_with(EngineVersion::new(3, 0, 0)));
    assert_eq!(CURRENT_ENGINE_VERSION.to_string(), "0.1.0");
    assert!(EngineVersion::from_str("2.14").is_err());
    assert!(EngineVersion::from_str("2.14.7.1").is_err());
}
