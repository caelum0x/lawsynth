//! Heat equation `u_t = α u_xx` from an exact two-mode analytic field.
//!
//! Because the field is an exact solution (no solver error), the only error is
//! the `O(dx²)`/`O(dt²)` truncation of the discovery's own finite differences.
//! A finer grid tightens the recovered `α`.

mod common;

use common::heat_two_mode;
use lawsynth_pde::{PdeConfig, discover_pde};

const ALPHA: f64 = 0.2;

fn heat_model(nx: usize, nt: usize) -> lawsynth_pde::PdeModel {
    // Modes k1 = 1, k2 = 2 break the single-mode u/u_xx collinearity.
    let fixture = heat_two_mode(ALPHA, 1.0, 2.0, 0.5, nx, nt, 0.01);
    discover_pde(&fixture.field, fixture.dx, fixture.dt, &PdeConfig::default()).unwrap()
}

#[test]
fn recovers_the_diffusion_coefficient() {
    let model = heat_model(96, 40);
    let coefficient = model.coefficient_of(0, 2);
    assert!((coefficient - ALPHA).abs() < 0.02, "u_xx coefficient {coefficient} not ≈ α = {ALPHA}");
}

#[test]
fn discovered_law_is_a_single_uxx_term() {
    let model = heat_model(96, 40);
    // Exactly one active term, and it is u_xx (u_power 0, derivative order 2).
    let active: Vec<_> = model.active_terms().collect();
    assert_eq!(active.len(), 1, "expected a single term, got {}", model.describe());
    assert_eq!(active[0].u_power, 0);
    assert_eq!(active[0].derivative_order, 2);
    assert_eq!(active[0].label, "u_xx");
}

#[test]
fn spurious_terms_are_negligible() {
    let model = heat_model(96, 40);
    // No advection, no reaction: u_x, u, u², u·u_x all ~ 0.
    assert!(model.coefficient_of(0, 1).abs() < 1e-3, "spurious u_x");
    assert!(model.coefficient_of(1, 0).abs() < 1e-3, "spurious u");
    assert!(model.coefficient_of(2, 0).abs() < 1e-3, "spurious u^2");
    assert!(model.coefficient_of(1, 1).abs() < 1e-3, "spurious u*u_x");
}

#[test]
fn finer_grid_tightens_recovery() {
    // Honest truncation claim: refining dx reduces the coefficient error.
    let coarse = (heat_model(48, 40).coefficient_of(0, 2) - ALPHA).abs();
    let fine = (heat_model(128, 40).coefficient_of(0, 2) - ALPHA).abs();
    assert!(fine <= coarse, "finer grid {fine} should not be worse than coarse {coarse}");
}

#[test]
fn residual_is_small_relative_to_the_dynamics() {
    let model = heat_model(96, 40);
    // A genuine fit: RSS per interior point is tiny next to the u_t scale.
    let mean_squared_residual = model.residual_sum_squares / model.interior_points as f64;
    assert!(mean_squared_residual < 1e-4, "mean squared residual {mean_squared_residual}");
}
