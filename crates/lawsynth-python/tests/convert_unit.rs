use std::collections::BTreeMap;

#[path = "../src/convert.rs"]
mod convert;

use convert::identifier_values;
use lawsynth_core::Identifier;

#[test]
fn converts_valid_python_mapping_to_sorted_typed_values() {
    let values = BTreeMap::from([("z".to_owned(), -2.5), ("alpha".to_owned(), 1.25)]);

    let converted = identifier_values(values).expect("finite identifiers should convert");

    assert_eq!(converted.len(), 2);
    assert_eq!(converted.keys().next().expect("first id").as_str(), "alpha");
    assert_eq!(converted[&Identifier::new("alpha").expect("valid identifier")], 1.25);
    assert_eq!(converted[&Identifier::new("z").expect("valid identifier")], -2.5);
}

#[test]
fn rejects_invalid_identifiers_and_non_finite_values() {
    let invalid_name = identifier_values(BTreeMap::from([("not valid".to_owned(), 1.0)]));
    assert!(invalid_name.is_err());

    let non_finite = identifier_values(BTreeMap::from([("x".to_owned(), f64::NAN)]));
    assert_eq!(non_finite.unwrap_err(), "value for 'x' must be finite");
}
