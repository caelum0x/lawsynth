//! The heart of the crate: each preset recovers its own reference law.
//!
//! For every preset we synthesize the reference trajectory, run discovery with
//! the preset's configuration and prior, and assert the discovered law matches
//! the reference law exactly in structure and to a tight coefficient tolerance.
//! The tolerance is deliberately far below the coefficient magnitudes (≈1) and
//! comfortably above the observed finite-difference error (≤1.3e-4), so a genuine
//! structural miss cannot slip through.

mod common;

use common::assert_round_trip;
use lawsynth_domains::preset;

/// Coefficient tolerance shared by every preset. The largest observed error is
/// ~1.3e-4 (the Lotka-Volterra prey growth term), so 1e-3 is tight yet honest.
const COEFFICIENT_TOLERANCE: f64 = 1e-3;

#[test]
fn damped_oscillator_recovers_its_reference_law() {
    // dx/dt = v ; dv/dt = -x - 0.5 v
    assert_round_trip(&preset("damped-oscillator").unwrap(), COEFFICIENT_TOLERANCE);
}

#[test]
fn lotka_volterra_recovers_its_reference_law() {
    // dprey/dt = 1.5 prey - prey predator ; dpredator/dt = 0.75 prey predator - predator
    assert_round_trip(&preset("lotka-volterra").unwrap(), COEFFICIENT_TOLERANCE);
}

#[test]
fn brusselator_recovers_its_reference_law() {
    // dx/dt = 1 - 4 x + x^2 y ; dy/dt = 3 x - x^2 y
    assert_round_trip(&preset("brusselator").unwrap(), COEFFICIENT_TOLERANCE);
}

#[test]
fn every_registered_preset_round_trips() {
    // Guards against a future preset being added to the registry without a
    // validating round-trip: this iterates the whole catalog.
    for preset in lawsynth_domains::all() {
        assert_round_trip(&preset, COEFFICIENT_TOLERANCE);
    }
}
