use lawsynth_profile::{profile_f64_missingness, profile_missingness};

#[test]
fn missingness_tracks_positions_runs_and_nonfinite_source_values() {
    let nullable = [Some(1.0), None, None, Some(4.0), None];
    let profile = profile_missingness(&nullable);
    assert_eq!(profile.total, 5);
    assert_eq!(profile.observed(), 2);
    assert_eq!(profile.missing_indices, vec![1, 2, 4]);
    assert_eq!(profile.longest_missing_run, 2);
    assert!((profile.fraction() - 0.6).abs() < f64::EPSILON);

    assert_eq!(
        profile_f64_missingness(&[1.0, f64::NAN, f64::INFINITY]).missing_indices,
        vec![1, 2]
    );
}
