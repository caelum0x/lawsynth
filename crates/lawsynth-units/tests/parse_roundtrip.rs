use lawsynth_units::{Unit, parse_unit};

#[test]
fn standard_unit_parser_retains_dimension_and_scale() {
    let parsed = parse_unit("km/min").unwrap();
    assert!(parsed.compatible_with(&Unit::parse("m/s").unwrap()));
    assert!((parsed.scale_to_si() - 50.0 / 3.0).abs() < 1e-12);
}
