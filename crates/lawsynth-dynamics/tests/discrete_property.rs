use lawsynth_dynamics::DynamicsConfig;

#[test]
fn dynamics_config_requires_two_or_more_samples() {
    assert!(DynamicsConfig { minimum_samples: 1 }.validate().is_err());
}
