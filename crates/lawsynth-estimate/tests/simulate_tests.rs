//! Integration tests for the estimator simulation — the demonstration that a
//! wrong initial estimate converges to the true state.

mod common;

use common::{damped_oscillator, diag2, double_integrator, real_poles, scalar};

use lawsynth_estimate::{
    EstimateError, MeasurementNoise, design_observer, kalman_filter, run_observer,
};

#[test]
fn placed_pole_observer_error_decays_to_zero() {
    // Double integrator, measure position only; place fast error poles {−4,−5}.
    let (a, b, c) = double_integrator();
    let observer = design_observer(&a, &c, &real_poles(&[-4.0, -5.0])).unwrap();

    let true_x0 = [1.0, 0.5];
    let est_x0 = [0.0, 0.0]; // deliberately wrong initial estimate
    let traj =
        run_observer(&observer, &a, &b, &c, &true_x0, &est_x0, &[], None, 0.01, 2000).unwrap();

    assert!(traj.initial_error() > 0.5, "should start with a real error");
    assert!(traj.final_error() < 1e-6, "error did not converge: final {}", traj.final_error());
    // Monotone-ish decay: the final error is far below the peak.
    assert!(traj.final_error() < 1e-3 * traj.max_error());
}

#[test]
fn kalman_observer_error_decays_measuring_only_position() {
    // Reconstruct the unmeasured velocity of a damped oscillator from position.
    let (a, b, c) = damped_oscillator();
    let observer = kalman_filter(&a, &c, &diag2(1.0, 1.0), &scalar(1e-4)).unwrap();

    let true_x0 = [1.0, 0.0];
    let est_x0 = [0.0, -1.0]; // wrong in both position and (unmeasured) velocity
    let traj =
        run_observer(&observer, &a, &b, &c, &true_x0, &est_x0, &[], None, 0.005, 4000).unwrap();

    assert!(traj.initial_error() > 0.9);
    assert!(
        traj.final_error() < 1e-5,
        "kalman error did not converge: final {}",
        traj.final_error()
    );

    // The velocity component (unmeasured) must itself be reconstructed.
    let final_true = traj.true_states.last().unwrap();
    let final_est = traj.estimates.last().unwrap();
    assert!((final_true[1] - final_est[1]).abs() < 1e-4);
}

#[test]
fn observer_tracks_a_forced_plant() {
    // Non-zero input: the observer must track the forced trajectory too.
    let (a, b, c) = double_integrator();
    let observer = design_observer(&a, &c, &real_poles(&[-3.0, -4.0])).unwrap();

    let steps = 1500;
    let inputs: Vec<Vec<f64>> = (0..steps).map(|k| vec![(k as f64 * 0.01).sin()]).collect();
    let true_x0 = [0.0, 0.0];
    let est_x0 = [2.0, -1.0];
    let traj =
        run_observer(&observer, &a, &b, &c, &true_x0, &est_x0, &inputs, None, 0.01, steps).unwrap();

    assert!(traj.initial_error() > 1.0);
    assert!(traj.final_error() < 1e-5, "forced-plant error {}", traj.final_error());
}

#[test]
fn noisy_kalman_estimate_stays_bounded_and_tracks() {
    let (a, b, c) = damped_oscillator();
    let observer = kalman_filter(&a, &c, &diag2(1e-3, 1e-3), &scalar(1e-2)).unwrap();

    let true_x0 = [1.0, 0.5];
    let est_x0 = [0.0, 0.0];
    let noise = MeasurementNoise::new(2024, 0.05);
    let traj =
        run_observer(&observer, &a, &b, &c, &true_x0, &est_x0, &[], Some(noise), 0.005, 4000)
            .unwrap();

    // The transient dies out; the steady error is small and bounded by the noise
    // level, not diverging.
    let tail_start = traj.errors.len() * 3 / 4;
    let tail_max = traj.errors[tail_start..].iter().copied().fold(0.0_f64, f64::max);
    assert!(tail_max < 0.1, "noisy tail error {tail_max} too large");
    assert!(traj.final_error() < traj.initial_error());
}

#[test]
fn simulation_is_bit_identical_across_runs() {
    let (a, b, c) = damped_oscillator();
    let observer = kalman_filter(&a, &c, &diag2(1e-3, 1e-3), &scalar(1e-2)).unwrap();
    let noise = MeasurementNoise::new(99, 0.03);

    let run = || {
        run_observer(&observer, &a, &b, &c, &[1.0, 0.5], &[0.0, 0.0], &[], Some(noise), 0.01, 500)
            .unwrap()
    };
    let first = run();
    let second = run();

    assert_eq!(first.errors.len(), second.errors.len());
    for (u, v) in first.errors.iter().zip(&second.errors) {
        assert_eq!(u.to_bits(), v.to_bits());
    }
    for (xa, xb) in first.estimates.iter().zip(&second.estimates) {
        for (ua, ub) in xa.iter().zip(xb) {
            assert_eq!(ua.to_bits(), ub.to_bits());
        }
    }
    for (ma, mb) in first.measurements.iter().zip(&second.measurements) {
        for (ua, ub) in ma.iter().zip(mb) {
            assert_eq!(ua.to_bits(), ub.to_bits());
        }
    }
}

#[test]
fn invalid_time_step_is_rejected() {
    let (a, b, c) = double_integrator();
    let observer = design_observer(&a, &c, &real_poles(&[-2.0, -3.0])).unwrap();
    let bad_dt = run_observer(&observer, &a, &b, &c, &[0.0, 0.0], &[1.0, 1.0], &[], None, 0.0, 10);
    assert_eq!(bad_dt.unwrap_err(), EstimateError::InvalidTimeStep);
    let zero_steps =
        run_observer(&observer, &a, &b, &c, &[0.0, 0.0], &[1.0, 1.0], &[], None, 0.01, 0);
    assert_eq!(zero_steps.unwrap_err(), EstimateError::InvalidTimeStep);
}

#[test]
fn input_signal_shape_mismatch_is_rejected() {
    let (a, b, c) = double_integrator();
    let observer = design_observer(&a, &c, &real_poles(&[-2.0, -3.0])).unwrap();
    // Wrong number of input samples (2, not 10).
    let inputs = vec![vec![0.0], vec![0.0]];
    let error =
        run_observer(&observer, &a, &b, &c, &[0.0, 0.0], &[1.0, 1.0], &inputs, None, 0.01, 10)
            .unwrap_err();
    assert_eq!(error, EstimateError::ShapeMismatch);
}

#[test]
fn initial_state_length_mismatch_is_rejected() {
    let (a, b, c) = double_integrator();
    let observer = design_observer(&a, &c, &real_poles(&[-2.0, -3.0])).unwrap();
    let error =
        run_observer(&observer, &a, &b, &c, &[0.0], &[1.0, 1.0], &[], None, 0.01, 10).unwrap_err();
    assert_eq!(error, EstimateError::ShapeMismatch);
}

#[test]
fn trajectory_lengths_are_consistent() {
    let (a, b, c) = double_integrator();
    let observer = design_observer(&a, &c, &real_poles(&[-2.0, -3.0])).unwrap();
    let steps = 50;
    let traj =
        run_observer(&observer, &a, &b, &c, &[1.0, 0.0], &[0.0, 0.0], &[], None, 0.02, steps)
            .unwrap();
    assert_eq!(traj.times.len(), steps + 1);
    assert_eq!(traj.true_states.len(), steps + 1);
    assert_eq!(traj.estimates.len(), steps + 1);
    assert_eq!(traj.measurements.len(), steps + 1);
    assert_eq!(traj.errors.len(), steps + 1);
    assert_eq!(traj.measurements[0].len(), 1); // p = 1 output
}
