use lawsynth_core::Identifier;

#[test]
fn identifiers_remain_orderable_and_preserve_portable_text() {
    let mut identifiers = [
        Identifier::new("velocity").unwrap(),
        Identifier::new("acceleration_1").unwrap(),
        Identifier::new("position").unwrap(),
    ];
    identifiers.sort();
    assert_eq!(
        identifiers
            .iter()
            .map(Identifier::as_str)
            .collect::<Vec<_>>(),
        ["acceleration_1", "position", "velocity"]
    );
}
