use lawsynth_uncertainty::{Samples, UncertaintyError};

#[test]
fn validated_samples_compute_unbiased_moments() {
    let samples = Samples::new(vec![1.0, 2.0, 3.0, 4.0]).unwrap();
    assert_eq!(samples.mean(), 2.5);
    assert!((samples.variance().unwrap() - 5.0 / 3.0).abs() < 1e-12);
    assert!(samples.standard_error().unwrap() > 0.0);
}

#[test]
fn samples_reject_empty_nonfinite_and_singleton_variance() {
    assert_eq!(
        Samples::new(vec![]).unwrap_err(),
        UncertaintyError::EmptyInput
    );
    assert_eq!(
        Samples::new(vec![f64::NAN]).unwrap_err(),
        UncertaintyError::NonFiniteValue
    );
    assert!(matches!(
        Samples::new(vec![1.0]).unwrap().variance(),
        Err(UncertaintyError::TooFewSamples { .. })
    ));
}
