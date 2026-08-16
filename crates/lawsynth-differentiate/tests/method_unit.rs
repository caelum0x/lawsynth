use lawsynth_differentiate::{DerivativeConfig, DerivativeMethod};

#[test]
fn default_configuration_selects_the_unparameterized_finite_method() {
    assert_eq!(
        DerivativeConfig::default().method,
        DerivativeMethod::FiniteDifference
    );
}

#[test]
fn configured_method_parameters_are_preserved_exactly() {
    let config = DerivativeConfig {
        method: DerivativeMethod::TotalVariation {
            lambda: 0.25,
            iterations: 64,
        },
    };
    assert_eq!(
        config.method,
        DerivativeMethod::TotalVariation {
            lambda: 0.25,
            iterations: 64,
        }
    );
}
