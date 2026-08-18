//! Determinism: identical `(field, dx, dt, config)` inputs yield a bit-identical
//! [`lawsynth_pde::PdeModel`].

mod common;

use common::{advection_two_mode, burgers_forward};
use lawsynth_pde::{PdeConfig, discover_pde};

#[test]
fn advection_discovery_is_bit_identical_across_runs() {
    let fixture = advection_two_mode(0.7, 1.0, 3.0, 0.4, 80, 30, 0.01);
    let config = PdeConfig::default();
    let first = discover_pde(&fixture.field, fixture.dx, fixture.dt, &config).unwrap();
    let second = discover_pde(&fixture.field, fixture.dx, fixture.dt, &config).unwrap();
    // Struct equality compares every coefficient bit-for-bit (f64: PartialEq).
    assert_eq!(first, second);
}

#[test]
fn burgers_discovery_is_bit_identical_across_runs() {
    let fixture = burgers_forward(0.12, 96, 40, 0.004, 6);
    let config = PdeConfig::default();
    let first = discover_pde(&fixture.field, fixture.dx, fixture.dt, &config).unwrap();
    let second = discover_pde(&fixture.field, fixture.dx, fixture.dt, &config).unwrap();

    assert_eq!(first.residual_sum_squares.to_bits(), second.residual_sum_squares.to_bits());
    for (a, b) in first.terms.iter().zip(&second.terms) {
        assert_eq!(a.coefficient.to_bits(), b.coefficient.to_bits(), "term {} differs", a.label);
    }
    assert_eq!(first, second);
}
