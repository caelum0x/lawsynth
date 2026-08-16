use lawsynth_wasm::Expression;
use std::collections::BTreeMap;
#[test]
fn parser_respects_precedence_functions_and_errors() {
    let expression = Expression::parse("2 + x * sin(t)^2").unwrap();
    let values = BTreeMap::from([("x".into(), 3.0), ("t".into(), std::f64::consts::FRAC_PI_2)]);
    assert!((expression.evaluate(&values).unwrap() - 5.0).abs() < 1e-12);
    assert!(Expression::parse("unknown(1)").is_err());
    assert!(Expression::parse("1 +").is_err());
}
