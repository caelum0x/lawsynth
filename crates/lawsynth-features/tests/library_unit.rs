use lawsynth_core::Identifier;
use lawsynth_features::{FeatureConfig, FeatureError, FeatureLibrary};

#[test]
fn configuration_defaults_to_a_quadratic_library_with_an_intercept() {
    assert_eq!(
        FeatureConfig::default(),
        FeatureConfig {
            polynomial_degree: 2,
            include_constant: true,
        }
    );
}

#[test]
fn constructors_reject_empty_and_duplicate_variable_sets() {
    assert_eq!(
        FeatureLibrary::polynomial([], 2, true),
        Err(FeatureError::EmptyVariables)
    );
    let x = Identifier::new("x").unwrap();
    assert_eq!(
        FeatureLibrary::interactions([x.clone(), x]),
        Err(FeatureError::DuplicateVariable("x".into()))
    );
}
