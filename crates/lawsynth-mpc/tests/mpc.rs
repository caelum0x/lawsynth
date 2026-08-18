//! Integration tests for successive-linearization MPC: regulation, saturation,
//! a discriminating baseline comparison, exact linear consistency, determinism,
//! and the boundary error paths.

mod common;

use common::{
    axpy, double_integrator, forced_pendulum, forced_van_der_pol, identity, linear_system, norm,
    rk4_rollout, scalar_weight,
};
use lawsynth_feedback::{FeedbackError, lqr};
use lawsynth_mpc::{MpcConfig, MpcError, mpc_control};

// ---------------------------------------------------------------------------
// Regulation
// ---------------------------------------------------------------------------

#[test]
fn regulates_forced_pendulum_to_origin() {
    let plant = forced_pendulum();
    let config =
        MpcConfig::new(vec![1.5, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(0.1), 0.02, 600);

    let trajectory = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();

    // The state is driven to the setpoint by the end of the horizon.
    let final_error = trajectory.final_error_norm(&[0.0, 0.0]).unwrap();
    assert!(final_error < 1e-3, "final error {final_error} not driven to setpoint");

    // Initial error was substantial, so real regulation happened.
    let initial_error = trajectory.error_norm(0, &[0.0, 0.0]).unwrap();
    assert!(initial_error > 1.0);
}

#[test]
fn regulated_state_stays_at_setpoint() {
    let plant = forced_pendulum();
    let config =
        MpcConfig::new(vec![1.0, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(0.1), 0.02, 800);

    let trajectory = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();

    // Once converged, every one of the last 50 samples remains near the setpoint.
    let total = trajectory.states().len();
    for step in (total - 50)..total {
        let error = trajectory.error_norm(step, &[0.0, 0.0]).unwrap();
        assert!(error < 1e-3, "state left the setpoint at step {step}: error {error}");
    }
}

#[test]
fn regulates_to_a_nonzero_setpoint() {
    // Pendulum has an equilibrium at (x = π, u = 0) — the inverted position — but
    // a shifted origin is easiest to check with the double integrator, which has
    // an equilibrium at any (x_ref, 0). Regulate the position to x = 2.
    let plant = double_integrator();
    let config =
        MpcConfig::new(vec![0.0, 0.0], vec![2.0, 0.0], identity(2), scalar_weight(1.0), 0.05, 400);

    let trajectory = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();
    assert!(trajectory.final_error_norm(&[2.0, 0.0]).unwrap() < 1e-3);
}

// ---------------------------------------------------------------------------
// Saturation
// ---------------------------------------------------------------------------

#[test]
fn respects_control_saturation() {
    let plant = double_integrator();
    let bound = 0.05;
    let config =
        MpcConfig::new(vec![1.0, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(0.01), 0.05, 400)
            .with_saturation(vec![-bound], vec![bound]);

    let trajectory = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();

    // Every applied control stays inside the tight bound.
    for control in trajectory.controls() {
        assert!(control[0] >= -bound - 1e-12 && control[0] <= bound + 1e-12, "u = {}", control[0]);
    }

    // And the controller still makes progress toward the setpoint.
    let initial = trajectory.error_norm(0, &[0.0, 0.0]).unwrap();
    let final_error = trajectory.final_error_norm(&[0.0, 0.0]).unwrap();
    assert!(final_error < initial, "no progress: {initial} -> {final_error}");
}

#[test]
fn saturation_makes_progress_slower_than_unconstrained() {
    let plant = double_integrator();
    let make = |sat: bool| {
        let base = MpcConfig::new(
            vec![1.0, 0.0],
            vec![0.0, 0.0],
            identity(2),
            scalar_weight(0.01),
            0.05,
            120,
        );
        if sat { base.with_saturation(vec![-0.05], vec![0.05]) } else { base }
    };

    let free = mpc_control(&plant.fields, &plant.states, &plant.controls, &make(false)).unwrap();
    let tight = mpc_control(&plant.fields, &plant.states, &plant.controls, &make(true)).unwrap();

    // Both are compared at the same (short) horizon: the constrained run has more
    // residual error because its actuator is throttled.
    let free_error = free.final_error_norm(&[0.0, 0.0]).unwrap();
    let tight_error = tight.final_error_norm(&[0.0, 0.0]).unwrap();
    assert!(tight_error > free_error, "constrained {tight_error} !> free {free_error}");
}

// ---------------------------------------------------------------------------
// Discriminating baseline: uncontrolled diverges, MPC stabilizes
// ---------------------------------------------------------------------------

#[test]
fn control_stabilizes_van_der_pol_while_baseline_reaches_limit_cycle() {
    let mu = 1.0;
    let plant = forced_van_der_pol(mu);
    let x0 = vec![0.5, 0.0];
    let dt = 0.01;
    let steps = 2000;

    // Uncontrolled reference (u = 0): the origin is unstable, so the state leaves
    // the neighbourhood and settles onto the limit cycle (amplitude ≈ 2).
    let baseline = rk4_rollout(
        |state| {
            let (x, y) = (state[0], state[1]);
            vec![y, mu * (1.0 - x * x) * y - x]
        },
        x0.clone(),
        dt,
        steps,
    );
    let baseline_final = norm(baseline.last().unwrap());
    assert!(baseline_final > 1.5, "baseline did not diverge to the limit cycle: {baseline_final}");

    // MPC with the same plant and initial state regulates to the origin.
    let config = MpcConfig::new(x0, vec![0.0, 0.0], identity(2), scalar_weight(0.5), dt, steps);
    let controlled = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();
    let controlled_final = controlled.final_error_norm(&[0.0, 0.0]).unwrap();
    assert!(controlled_final < 1e-3, "MPC failed to stabilize: {controlled_final}");

    // The comparison is genuinely discriminating.
    assert!(controlled_final < baseline_final);
}

#[test]
fn nonlinear_gain_schedule_varies_across_steps() {
    // For a nonlinear plant the linearization changes with the state, so the
    // per-step LQR gains are not all identical (unlike the linear case below).
    let plant = forced_van_der_pol(1.0);
    let config =
        MpcConfig::new(vec![1.5, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(0.5), 0.01, 400);
    let trajectory = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();

    let first = &trajectory.gains()[0];
    let differs = trajectory
        .gains()
        .iter()
        .any(|gain| (0..gain.cols()).any(|j| (gain.get(0, j) - first.get(0, j)).abs() > 1e-6));
    assert!(differs, "gain schedule was constant for a nonlinear plant");
}

// ---------------------------------------------------------------------------
// Linear consistency: one-step linearization is exact
// ---------------------------------------------------------------------------

#[test]
fn linear_plant_gain_equals_direct_lqr() {
    let a = [[0.0, 1.0], [-1.0, -0.5]];
    let b = [[0.0], [1.0]];
    let (plant, a_matrix, b_matrix) = linear_system(a, b);
    let q = identity(2);
    let r = scalar_weight(1.0);

    let config = MpcConfig::new(vec![1.0, 0.5], vec![0.0, 0.0], q.clone(), r.clone(), 0.02, 50);
    let trajectory = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();

    // The linearization of a linear plant is exact, so every step designs the
    // same gain, and it is bit-identical to a direct LQR solve of (A, B, Q, R).
    let direct = lqr(&a_matrix, &b_matrix, &q, &r).unwrap();
    for gain in trajectory.gains() {
        for i in 0..gain.rows() {
            for j in 0..gain.cols() {
                assert_eq!(gain.get(i, j).to_bits(), direct.k.get(i, j).to_bits());
            }
        }
    }
}

#[test]
fn linear_closed_loop_matches_lqr_rollout() {
    let a = [[0.0, 1.0], [-2.0, -0.3]];
    let b = [[0.0], [1.0]];
    let (plant, a_matrix, b_matrix) = linear_system(a, b);
    let q = identity(2);
    let r = scalar_weight(0.5);
    let dt = 0.02;
    let steps = 800;
    let x0 = vec![1.0, -0.5];

    let config = MpcConfig::new(x0.clone(), vec![0.0, 0.0], q.clone(), r.clone(), dt, steps);
    let mpc = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();

    // Independent rollout of ẋ = A x + B u with the fixed LQR law u = −K x. As in
    // the controller, the move is computed once from the step-start state and held
    // constant across the RK4 stages.
    let gain = lqr(&a_matrix, &b_matrix, &q, &r).unwrap().k;
    let plant_derivative = |state: &[f64], control: f64| {
        let mut derivative = a_matrix.mat_vec(state).unwrap();
        let forcing = b_matrix.mat_vec(&[control]).unwrap();
        for (d, f) in derivative.iter_mut().zip(forcing) {
            *d += f;
        }
        derivative
    };
    let mut state = x0;
    let mut reference = vec![state.clone()];
    for _ in 0..steps {
        let control = -gain.mat_vec(&state).unwrap()[0];
        let deriv = |x: &[f64]| plant_derivative(x, control);
        let k1 = deriv(&state);
        let k2 = deriv(&axpy(&state, &k1, dt / 2.0));
        let k3 = deriv(&axpy(&state, &k2, dt / 2.0));
        let k4 = deriv(&axpy(&state, &k3, dt));
        state = state
            .iter()
            .enumerate()
            .map(|(i, value)| value + (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]))
            .collect();
        reference.push(state.clone());
    }

    for (mpc_state, ref_state) in mpc.states().iter().zip(&reference) {
        for (a_value, b_value) in mpc_state.iter().zip(ref_state) {
            assert!((a_value - b_value).abs() < 1e-9, "mpc {a_value} vs ref {b_value}");
        }
    }
    assert!(mpc.final_error_norm(&[0.0, 0.0]).unwrap() < 1e-3);
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn identical_inputs_yield_bit_identical_trajectory() {
    let plant = forced_van_der_pol(0.8);
    let config =
        MpcConfig::new(vec![0.7, -0.2], vec![0.0, 0.0], identity(2), scalar_weight(0.3), 0.01, 500);

    let first = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();
    let second = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap();

    assert_eq!(first.bit_fingerprint(), second.bit_fingerprint());
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

#[test]
fn rejects_setpoint_dimension_mismatch() {
    let plant = double_integrator();
    let config =
        MpcConfig::new(vec![0.0, 0.0], vec![0.0], identity(2), scalar_weight(1.0), 0.05, 10);
    let error = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap_err();
    assert!(matches!(
        error,
        MpcError::DimensionMismatch { what: "setpoint", expected: 2, actual: 1 }
    ));
}

#[test]
fn propagates_non_positive_definite_r() {
    let plant = double_integrator();
    let config =
        MpcConfig::new(vec![1.0, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(-1.0), 0.05, 10);
    let error = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap_err();
    assert_eq!(error, MpcError::Design(FeedbackError::NotPositiveDefinite));
}

#[test]
fn rejects_empty_controls() {
    let plant = double_integrator();
    let config =
        MpcConfig::new(vec![0.0, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(1.0), 0.05, 10);
    let error = mpc_control(&plant.fields, &plant.states, &[], &config).unwrap_err();
    assert_eq!(error, MpcError::EmptyControls);
}

#[test]
fn rejects_non_finite_setpoint() {
    let plant = double_integrator();
    let config = MpcConfig::new(
        vec![0.0, 0.0],
        vec![f64::NAN, 0.0],
        identity(2),
        scalar_weight(1.0),
        0.05,
        10,
    );
    let error = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap_err();
    assert_eq!(error, MpcError::NonFiniteConfig("setpoint"));
}

#[test]
fn rejects_empty_horizon() {
    let plant = double_integrator();
    let config =
        MpcConfig::new(vec![0.0, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(1.0), 0.05, 0);
    let error = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap_err();
    assert_eq!(error, MpcError::EmptyHorizon);
}

#[test]
fn rejects_non_positive_time_step() {
    let plant = double_integrator();
    let config =
        MpcConfig::new(vec![0.0, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(1.0), 0.0, 10);
    let error = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap_err();
    assert!(matches!(error, MpcError::InvalidTimeStep(dt) if dt == 0.0));
}

#[test]
fn rejects_inconsistent_saturation_bounds() {
    let plant = double_integrator();
    let config =
        MpcConfig::new(vec![1.0, 0.0], vec![0.0, 0.0], identity(2), scalar_weight(1.0), 0.05, 10)
            .with_saturation(vec![0.5], vec![-0.5]);
    let error = mpc_control(&plant.fields, &plant.states, &plant.controls, &config).unwrap_err();
    assert_eq!(error, MpcError::InvalidSaturation { index: 0 });
}

#[test]
fn rejects_missing_field_for_a_state() {
    // A state with no field cannot be linearized.
    let plant = double_integrator();
    let extra = common::id("z");
    let mut states = plant.states.clone();
    states.push(extra);
    let config = MpcConfig::new(
        vec![0.0, 0.0, 0.0],
        vec![0.0, 0.0, 0.0],
        identity(3),
        scalar_weight(1.0),
        0.05,
        10,
    );
    let error = mpc_control(&plant.fields, &states, &plant.controls, &config).unwrap_err();
    assert!(matches!(error, MpcError::Linearization(_)));
}
