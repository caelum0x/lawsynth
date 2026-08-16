use lawsynth_data::TimeAxis;
use lawsynth_profile::TimeProfile;

#[test]
fn time_profile_distinguishes_regular_and_irregular_axes_at_configured_tolerance() {
    let regular = TimeAxis::new(vec![3.0, 3.5, 4.0, 4.5]).unwrap();
    let irregular = TimeAxis::new(vec![3.0, 3.5, 4.1, 4.5]).unwrap();
    let result = TimeProfile::from_time_axis(&regular);
    assert_eq!(result.start, 3.0);
    assert_eq!(result.end, 4.5);
    assert_eq!(result.nominal_step, 0.5);
    assert!(result.is_regular);
    assert!(!TimeProfile::from_time_axis(&irregular).is_regular);
}
