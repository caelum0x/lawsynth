use lawsynth_stats::{StatsError, moments};

#[test]
fn welford_moments_report_population_and_sample_variance() {
    let summary = moments(&[1.0, 2.0, 3.0]).unwrap();
    assert_eq!(summary.count, 3);
    assert_eq!(summary.mean, 2.0);
    assert!((summary.population_variance - 2.0 / 3.0).abs() < 1e-12);
    assert_eq!(summary.sample_variance, 1.0);
    assert_eq!(moments(&[f64::NAN]), Err(StatsError::NonFiniteValue));
}
