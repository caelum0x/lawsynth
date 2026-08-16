use lawsynth_core::Seed;
use lawsynth_stats::{
    HistogramConfig, StatsError, histogram_mutual_information, normal_cdf, normal_pdf,
    sample_without_replacement,
};

#[test]
fn normal_distribution_primitives_are_numerically_sane() {
    assert!((normal_cdf(0.0, 0.0, 1.0).unwrap() - 0.5).abs() < 1e-7);
    assert!((normal_pdf(0.0, 0.0, 1.0).unwrap() - 0.398_942_280_4).abs() < 1e-9);
    assert_eq!(
        normal_pdf(0.0, 0.0, 0.0),
        Err(StatsError::InvalidStandardDeviation)
    );
}

#[test]
fn sampling_and_information_are_deterministic_and_validate_inputs() {
    let first = sample_without_replacement(10, 4, Seed::new(7)).unwrap();
    assert_eq!(
        first,
        sample_without_replacement(10, 4, Seed::new(7)).unwrap()
    );
    assert_eq!(first.len(), 4);
    assert_eq!(
        sample_without_replacement(2, 3, Seed::new(0)),
        Err(StatsError::SampleExceedsPopulation)
    );

    let information = histogram_mutual_information(
        &[0.0, 1.0, 2.0, 3.0],
        &[0.0, 1.0, 2.0, 3.0],
        HistogramConfig { bins: 2 },
    )
    .unwrap();
    assert!(information > 0.6);
}
