use lawsynth_core::Identifier;
use lawsynth_discovery::{DiscoveryConfig, DiscoveryPlan, DiscoveryStage};

#[test]
fn plan_retains_requested_states_and_the_complete_execution_stage_order() {
    let config =
        DiscoveryConfig::new([Identifier::new("x").unwrap(), Identifier::new("y").unwrap()]);
    let plan = DiscoveryPlan::from_config(&config);
    assert_eq!(plan.states, config.state);
    assert_eq!(plan.stages, DiscoveryStage::all());
    assert!(!plan.is_empty());
    assert!(DiscoveryPlan::from_config(&DiscoveryConfig::new([])).is_empty());
}
