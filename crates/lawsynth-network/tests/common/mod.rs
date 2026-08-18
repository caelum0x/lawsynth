#![allow(dead_code)]
//! Shared deterministic coupled-RK4 fixture for the network recovery tests.
//!
//! Every test integrates a *known* coupling graph with a fixed-step classical
//! RK4 integrator, assembles the trajectory into a [`Dataset`], and then asks
//! `discover_network` to recover the graph. The integrator is a pure `f64`
//! computation, so the generated data — and therefore the discovered model — is
//! reproducible.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// Integrates `ẋ = f(x)` for `steps` fixed steps of size `dt` from `x0`.
///
/// Returns the time samples (`steps + 1` of them) and, for each state, its full
/// sampled trajectory in `state[node][sample]` layout.
pub fn integrate_rk4<F>(x0: &[f64], dt: f64, steps: usize, f: F) -> (Vec<f64>, Vec<Vec<f64>>)
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    let n = x0.len();
    let mut time = Vec::with_capacity(steps + 1);
    let mut trajectory = vec![Vec::with_capacity(steps + 1); n];
    let mut state = x0.to_vec();

    for step in 0..=steps {
        time.push(step as f64 * dt);
        for (node, value) in state.iter().enumerate() {
            trajectory[node].push(*value);
        }
        if step == steps {
            break;
        }
        state = rk4_step(&state, dt, &f);
    }

    (time, trajectory)
}

/// Integrates several trajectories from distinct initial conditions and stitches
/// them into one dataset-ready trajectory.
///
/// A single trajectory of a symmetric linear network is *not* persistently
/// exciting — degenerate Laplacian eigenvalues collapse the reachable subspace,
/// so the design matrix is rank-deficient and the coupling is unidentifiable.
/// Exciting the network from several initial conditions restores full rank, the
/// standard SINDy remedy.
///
/// Segments are separated on the time axis by a large `gap`. The interior
/// three-point derivative used downstream then degrades, at each junction, to a
/// correct *within-segment* one-sided difference (its `gap → ∞` limit), so
/// stitching introduces no cross-segment derivative corruption.
pub fn integrate_multi<F>(
    initial_conditions: &[Vec<f64>],
    dt: f64,
    steps: usize,
    gap: f64,
    f: F,
) -> (Vec<f64>, Vec<Vec<f64>>)
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    assert!(!initial_conditions.is_empty(), "need at least one initial condition");
    let nodes = initial_conditions[0].len();
    let mut time = Vec::new();
    let mut trajectory = vec![Vec::new(); nodes];
    let mut base = 0.0;

    for x0 in initial_conditions {
        assert_eq!(x0.len(), nodes, "every initial condition needs the same node count");
        let (segment_time, segment_states) = integrate_rk4(x0, dt, steps, &f);
        for stamp in &segment_time {
            time.push(base + stamp);
        }
        for (node, values) in segment_states.into_iter().enumerate() {
            trajectory[node].extend(values);
        }
        base = *time.last().expect("segment produced samples") + gap;
    }

    (time, trajectory)
}

fn rk4_step<F>(state: &[f64], dt: f64, f: &F) -> Vec<f64>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    let n = state.len();
    let k1 = f(state);
    let s2: Vec<f64> = (0..n).map(|i| state[i] + 0.5 * dt * k1[i]).collect();
    let k2 = f(&s2);
    let s3: Vec<f64> = (0..n).map(|i| state[i] + 0.5 * dt * k2[i]).collect();
    let k3 = f(&s3);
    let s4: Vec<f64> = (0..n).map(|i| state[i] + dt * k3[i]).collect();
    let k4 = f(&s4);
    (0..n).map(|i| state[i] + dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i])).collect()
}

/// Builds a [`Dataset`] from a simulated trajectory and per-node names.
///
/// Panics on malformed input — this is test-only scaffolding.
pub fn dataset_from(time: Vec<f64>, names: &[&str], trajectory: Vec<Vec<f64>>) -> Dataset {
    assert_eq!(names.len(), trajectory.len(), "one name per node");
    let columns = names
        .iter()
        .zip(trajectory)
        .map(|(name, values)| NumericColumn::new(Identifier::new(*name).unwrap(), values))
        .collect::<Vec<_>>();
    Dataset::new(TimeAxis::new(time).unwrap(), columns).unwrap()
}

/// Convenience: `Identifier` from a `&str` for terse assertions.
pub fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}
