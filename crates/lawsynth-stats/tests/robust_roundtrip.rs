use lawsynth_stats::{median_absolute_deviation, winsorize};

#[test]
fn robust_statistics_resist_a_single_extreme_observation() {
    assert_eq!(
        median_absolute_deviation(&[1.0, 2.0, 3.0, 100.0]).unwrap(),
        1.0
    );
    assert_eq!(
        winsorize(&[1.0, 2.0, 3.0, 100.0], 0.25).unwrap(),
        vec![1.75, 2.0, 3.0, 27.25]
    );
}
