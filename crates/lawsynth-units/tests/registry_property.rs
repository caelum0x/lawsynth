use lawsynth_units::{Dimension, Unit, UnitError, UnitRegistry, convert};

#[test]
fn custom_registry_units_preserve_dimension_and_scale_through_composition() {
    let mut registry = UnitRegistry::default();
    registry.register("cm", Unit::from_parts("cm", Dimension::LENGTH, 0.01).unwrap()).unwrap();
    let speed = registry.parse("cm/s").unwrap();
    assert!((convert(250.0, &speed, &Unit::parse("m/s").unwrap()).unwrap() - 2.5).abs() < 1e-12);
    assert_eq!(
        registry.register("cm", Unit::parse("m").unwrap()),
        Err(UnitError::DuplicateUnit("cm".into()))
    );
}
