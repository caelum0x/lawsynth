use lawsynth_uncertainty::{
    BootstrapConfig, IntervalConfig, ProfilePoint, PropagationConfig, Samples, bootstrap,
    confidence_interval, monte_carlo_propagate, profile_quadratic,
};

#[test]
fn deterministic_bootstrap_has_a_finite_percentile_interval() {
    let samples = Samples::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
    let config = BootstrapConfig {
        replicates: 512,
        seed: 41,
    };
    let first = bootstrap(&samples, config, |draw| {
        draw.iter().sum::<f64>() / draw.len() as f64
    })
    .unwrap();
    let second = bootstrap(&samples, config, |draw| {
        draw.iter().sum::<f64>() / draw.len() as f64
    })
    .unwrap();
    assert_eq!(first, second);
    let (lower, upper) = confidence_interval(&first, IntervalConfig { confidence: 0.95 }).unwrap();
    assert!(lower <= first.observed && first.observed <= upper);
    assert!(first.standard_error().unwrap() > 0.0);
}

#[test]
fn empirical_propagation_and_profile_fit_use_real_observations() {
    let input = Samples::new(vec![1.0, 2.0, 3.0]).unwrap();
    let propagated = monte_carlo_propagate(
        &[input],
        PropagationConfig {
            draws: 128,
            seed: 8,
        },
        |values| values[0] * 2.0,
    )
    .unwrap();
    assert_eq!(propagated.len(), 128);
    assert!(
        propagated
            .as_slice()
            .iter()
            .all(|value| [2.0, 4.0, 6.0].contains(value))
    );

    let points: Vec<ProfilePoint> = [-1.0, 0.0, 1.0, 2.0, 3.0]
        .into_iter()
        .map(|parameter| ProfilePoint {
            parameter,
            objective: 3.0 * (parameter - 1.5).powi(2) + 2.0,
        })
        .collect();
    let profile = profile_quadratic(&points, IntervalConfig::default()).unwrap();
    assert!((profile.optimum - 1.5).abs() < 1e-12);
    assert!((profile.minimum - 2.0).abs() < 1e-12);
    assert!(profile.interval.0 < profile.optimum && profile.interval.1 > profile.optimum);
}
