use lawsynth_units::{Unit, UnitError, builtin_registry, require_compatible};

#[test]
fn registry_and_dimension_check_support_standard_units() {
    let registry = builtin_registry();
    let velocity = registry.parse("km/min").unwrap();
    require_compatible(&velocity, &Unit::parse("m/s").unwrap()).unwrap();
    assert_eq!(
        require_compatible(&velocity, &Unit::parse("kg").unwrap()),
        Err(UnitError::IncompatibleDimensions)
    );
}
