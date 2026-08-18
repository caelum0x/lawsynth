//! Burgers' equation `u_t = ν u_xx − u u_x` from a stable RK4 forward solve.
//!
//! The headline recovery: BOTH the viscous `u_xx` term (`≈ ν`) and the advective
//! nonlinearity `u·u_x` (`≈ −1`) must be found. The nonlinear cascade fills the
//! spectrum, so `u`/`u_xx` are well separated (no single-mode degeneracy).

mod common;

use common::burgers_forward;
use lawsynth_pde::{PdeConfig, discover_pde};

const NU: f64 = 0.1;

fn burgers_model() -> lawsynth_pde::PdeModel {
    // 128 spatial points, 80 snapshots at dt = 0.004 with 8 RK4 substeps each.
    let fixture = burgers_forward(NU, 128, 80, 0.004, 8);
    discover_pde(&fixture.field, fixture.dx, fixture.dt, &PdeConfig::default()).unwrap()
}

#[test]
fn recovers_the_viscosity() {
    let model = burgers_model();
    let nu = model.coefficient_of(0, 2);
    assert!((nu - NU).abs() < 0.02, "u_xx coefficient {nu} not ≈ ν = {NU}");
}

#[test]
fn recovers_the_advective_nonlinearity() {
    let model = burgers_model();
    let coefficient = model.coefficient_of(1, 1);
    assert!((coefficient - (-1.0)).abs() < 0.05, "u*u_x coefficient {coefficient} not ≈ −1");
}

#[test]
fn discovers_exactly_the_two_burgers_terms() {
    let model = burgers_model();
    let mut active: Vec<&str> = model.active_terms().map(|term| term.label.as_str()).collect();
    active.sort_unstable();
    assert_eq!(active, vec!["u*u_x", "u_xx"], "law was {}", model.describe());
}

#[test]
fn spurious_terms_are_negligible() {
    let model = burgers_model();
    assert!(model.coefficient_of(0, 0).abs() < 1e-2, "spurious constant");
    assert!(model.coefficient_of(1, 0).abs() < 1e-2, "spurious u");
    assert!(model.coefficient_of(0, 1).abs() < 1e-2, "spurious u_x");
}
