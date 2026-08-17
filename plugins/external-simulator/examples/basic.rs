//! Integrate a 1-D linear decay ODE with the reference simulator core.
//!
//! ```bash
//! cargo run --example basic
//! ```
//!
//! The same `LinearSimulator` used here is the compute core a process worker
//! wraps behind the frame protocol described in `docs/usage.md`.

use lawsynth_external_simulator::LinearSimulator;
use lawsynth_plugin_api::{SimulationPlugin, SimulationRequest};

fn main() {
    // dx/dt = -x  (exponential decay toward zero).
    let simulator = LinearSimulator::new(vec![vec![-1.0]], vec![0.0], 0.001)
        .expect("a 1x1 matrix with a positive step is valid");

    let request = SimulationRequest {
        initial_state: vec![1.0],
        times: vec![0.0, 0.5, 1.0, 2.0],
    };

    let response = simulator
        .simulate(request)
        .expect("simulation should converge");

    for (state, time) in response.states.iter().zip([0.0, 0.5, 1.0, 2.0]) {
        println!("t = {time:>4}:  x = {:.6}", state[0]);
    }
}
