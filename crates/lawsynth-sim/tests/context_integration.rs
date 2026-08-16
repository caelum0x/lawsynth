use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_sim::SimulationContext;

#[test]
fn context_builds_a_complete_expression_environment() {
    let id = |name| Identifier::new(name).unwrap();
    let context = SimulationContext::new(
        BTreeMap::from([(id("x"), 1.0)]),
        BTreeMap::from([(id("rate"), 2.0)]),
        BTreeMap::from([(id("u"), 3.0)]),
    );
    let environment = context.environment();
    assert_eq!(environment[&id("x")], 1.0);
    assert_eq!(environment[&id("rate")], 2.0);
    assert_eq!(environment[&id("u")], 3.0);
}
