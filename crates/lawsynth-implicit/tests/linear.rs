//! Linear-system sanity: implicit discovery must degrade gracefully to ordinary
//! explicit dynamics when the denominator is a constant.

mod support;

use lawsynth_implicit::{ImplicitConfig, implicit_discover};
use support::{coefficient, dataset_x, integrate, linear_decay};

#[test]
fn recovers_linear_decay_with_constant_denominator() {
    let k = 0.8;
    let (time, xs) = integrate(linear_decay(k), 1.0, 0.01, 400);
    let dataset = dataset_x(time, xs);

    let config = ImplicitConfig { degree: 2, ..Default::default() };
    let result = implicit_discover(&dataset, &config).unwrap();

    assert!(result.relation.consistent);
    let law = result.rational_law.expect("rational law reconstructed");

    // ẋ = -k·x means Q(x) = 1 (constant) and P(x) = -k·x.
    assert!((coefficient(&law.denominator.terms, "1") - 1.0).abs() < 1e-9);
    assert!(coefficient(&law.denominator.terms, "x").abs() < 1e-6);
    let recovered_k = -coefficient(&law.numerator.terms, "x");
    assert!((recovered_k - k).abs() < 5e-3, "k recovered {recovered_k}");

    // A constant, non-vanishing denominator is the ordinary explicit case.
    assert!(law.denominator_nonvanishing);
    assert!((law.min_abs_denominator - 1.0).abs() < 1e-9);
}
