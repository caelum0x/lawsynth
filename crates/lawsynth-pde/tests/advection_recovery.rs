//! Advection `u_t = −c u_x` from an exact two-mode travelling wave.
//!
//! The clean linear check: the discovery must pick the single `u_x` term with
//! coefficient `≈ −c`.

mod common;

use common::advection_two_mode;
use lawsynth_pde::{PdeConfig, discover_pde};

const C: f64 = 0.8;

fn advection_model(nx: usize, nt: usize) -> lawsynth_pde::PdeModel {
    let fixture = advection_two_mode(C, 1.0, 2.0, 0.5, nx, nt, 0.01);
    discover_pde(&fixture.field, fixture.dx, fixture.dt, &PdeConfig::default()).unwrap()
}

#[test]
fn recovers_the_advection_speed() {
    let model = advection_model(96, 40);
    let coefficient = model.coefficient_of(0, 1);
    assert!((coefficient - (-C)).abs() < 0.02, "u_x coefficient {coefficient} not ≈ −c = {}", -C);
}

#[test]
fn discovered_law_is_a_single_ux_term() {
    let model = advection_model(96, 40);
    let active: Vec<_> = model.active_terms().collect();
    assert_eq!(active.len(), 1, "expected a single term, got {}", model.describe());
    assert_eq!(active[0].label, "u_x");
    assert_eq!(active[0].u_power, 0);
    assert_eq!(active[0].derivative_order, 1);
}

#[test]
fn spurious_terms_are_negligible() {
    let model = advection_model(96, 40);
    assert!(model.coefficient_of(0, 2).abs() < 1e-3, "spurious u_xx");
    assert!(model.coefficient_of(1, 0).abs() < 1e-3, "spurious u");
    assert!(model.coefficient_of(1, 1).abs() < 1e-3, "spurious u*u_x");
}
