use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::parse;
use lawsynth_sim::{SimulationConfig, simulate};
use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole, World};

#[path = "../src/convert.rs"]
mod convert;
pub use convert::identifier_values;
#[path = "../src/py_run.rs"]
mod py_run;
#[path = "../src/py_simulation.rs"]
mod py_simulation;

use py_run::request_from_values;
use py_simulation::trajectory_values;

fn id(value: &str) -> Identifier {
    Identifier::new(value).expect("test id is valid")
}

#[test]
fn python_value_maps_drive_a_real_simulation_and_return_string_keyed_values() {
    let x = id("x");
    let u = id("u");
    let rate = id("rate");
    let world = World::new(
        [
            Variable::new(x.clone(), VariableRole::State),
            Variable::new(u, VariableRole::Control),
        ],
        [Parameter::new(rate, 0.5)],
        [ContinuousLaw::new(
            x.clone(),
            parse("rate * x + u").expect("valid law"),
        )],
    )
    .expect("valid world");
    let request = request_from_values(
        BTreeMap::from([("x".to_owned(), 2.0)]),
        BTreeMap::from([("rate".to_owned(), 1.0)]),
        BTreeMap::from([("u".to_owned(), 3.0)]),
    )
    .expect("finite Python mappings should become a request");

    assert_eq!(request.initial_state[&x], 2.0);
    assert_eq!(request.parameter_overrides[&id("rate")], 1.0);
    assert_eq!(request.inputs[&id("u")], 3.0);

    let trajectory = simulate(
        &world,
        SimulationConfig::new(0.0, 0.5, 0.1).expect("valid grid"),
        &request,
    )
    .expect("request should execute against the world");
    let values = trajectory_values(&trajectory);

    assert_eq!(trajectory.time.first(), Some(&0.0));
    assert_eq!(values["x"].first(), Some(&2.0));
    assert_eq!(values["x"].len(), trajectory.samples());
    assert!(values["x"].last().expect("simulation has samples") > &2.0);
}

#[test]
fn non_finite_python_values_never_reach_the_simulator() {
    let result = request_from_values(
        BTreeMap::from([("x".to_owned(), f64::INFINITY)]),
        BTreeMap::new(),
        BTreeMap::new(),
    );

    assert_eq!(result.unwrap_err(), "value for 'x' must be finite");
}
