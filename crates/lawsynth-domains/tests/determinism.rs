//! Determinism: trajectories are bit-identical and discovery is reproducible.

mod common;

use common::recovered_laws;
use lawsynth_domains::{all, preset};

/// Two trajectory generations of the same preset must be bit-for-bit identical
/// across every channel — the RK4 integrator reads no clock and no randomness.
#[test]
fn trajectories_are_bit_identical_across_calls() {
    for preset in all() {
        let first = preset.reference().trajectory();
        let second = preset.reference().trajectory();
        for state in preset.state_variables() {
            let a = &first.columns()[state].values;
            let b = &second.columns()[state].values;
            assert_eq!(a.len(), b.len(), "channel length differs for {}", preset.name());
            assert!(
                a.iter().zip(b).all(|(left, right)| left.to_bits() == right.to_bits()),
                "trajectory not bit-identical for {} channel {state}",
                preset.name(),
            );
        }
    }
}

/// The trajectory has exactly `steps + 1` samples with the expected uniform time
/// axis, independent of the domain.
#[test]
fn trajectory_length_matches_the_reference_schedule() {
    let brusselator = preset("brusselator").unwrap();
    let data = brusselator.reference().trajectory();
    assert_eq!(data.time().len(), brusselator.reference().steps() + 1);
    for state in brusselator.state_variables() {
        assert_eq!(data.columns()[state].values.len(), brusselator.reference().steps() + 1);
    }
}

/// Running discovery twice on the same preset yields identical recovered laws.
#[test]
fn discovery_is_reproducible() {
    let lotka = preset("lotka-volterra").unwrap();
    assert_eq!(recovered_laws(&lotka), recovered_laws(&lotka));
}

/// The reference initial condition seeds the very first trajectory sample.
#[test]
fn first_sample_is_the_initial_condition() {
    for preset in all() {
        let data = preset.reference().trajectory();
        for (index, state) in preset.state_variables().iter().enumerate() {
            let first = data.columns()[state].values[0];
            assert_eq!(first.to_bits(), preset.reference().initial()[index].to_bits());
        }
    }
}
