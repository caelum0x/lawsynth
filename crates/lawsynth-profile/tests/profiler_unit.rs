use lawsynth_profile::{ProfileConfig, ProfileError};

#[test]
fn profiling_configuration_rejects_negative_or_nonfinite_tolerance() {
    assert_eq!(
        ProfileConfig { regularity_tolerance: -0.1 }.validate(),
        Err(ProfileError::InvalidConfiguration)
    );
    assert_eq!(
        ProfileConfig { regularity_tolerance: f64::NAN }.validate(),
        Err(ProfileError::InvalidConfiguration)
    );
}
