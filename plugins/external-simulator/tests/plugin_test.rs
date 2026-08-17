use lawsynth_external_simulator::LinearSimulator;
use lawsynth_plugin_api::{PluginError, SimulationPlugin, SimulationRequest};

#[test]
fn integrates_linear_decay() {
    let simulator = LinearSimulator::new(vec![vec![-1.0]], vec![0.0], 0.001).unwrap();
    let request = SimulationRequest {
        initial_state: vec![1.0],
        times: vec![0.0, 1.0],
    };
    let response = simulator.simulate(request).unwrap();

    assert_eq!(response.states.len(), 2);
    assert_eq!(response.states[0], vec![1.0]);
    // Explicit-Euler estimate of e^-1 ≈ 0.3679.
    let final_state = response.states[1][0];
    assert!(
        (0.30..0.40).contains(&final_state),
        "final state was {final_state}"
    );
}

#[test]
fn rejects_non_square_matrix() {
    let error = LinearSimulator::new(vec![vec![-1.0, 0.0]], vec![0.0], 0.01).unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}

#[test]
fn rejects_initial_state_width_mismatch() {
    let simulator = LinearSimulator::new(vec![vec![-1.0]], vec![0.0], 0.01).unwrap();
    let request = SimulationRequest {
        initial_state: vec![1.0, 2.0],
        times: vec![0.0, 1.0],
    };
    let error = simulator.simulate(request).unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}
