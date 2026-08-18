//! Shared fixtures for the stability integration tests.
//!
//! Each builder returns `(fields, states)` for a small vector field with a known
//! fixed-point structure, so the tests can assert both the located coordinates
//! and the classification.

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};

/// A validated identifier (panics on an invalid name — test-only convenience).
pub fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// `-expr`.
pub fn neg(expr: Expr) -> Expr {
    Expr::unary(UnaryOperator::Negate, expr)
}

/// `sin(expr)`.
pub fn sin(expr: Expr) -> Expr {
    Expr::unary(UnaryOperator::Sin, expr)
}

/// The symbol named `name`.
pub fn sym(name: &str) -> Expr {
    Expr::symbol(id(name))
}

/// Two states `x, y`.
pub fn xy() -> Vec<Identifier> {
    vec![id("x"), id("y")]
}

/// Stable node: `x' = -x`, `y' = -2y`. One fixed point at the origin.
pub fn stable_node() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields =
        vec![(id("x"), neg(sym("x"))), (id("y"), Expr::product(Expr::constant(-2.0), sym("y")))];
    (fields, xy())
}

/// Center (harmonic oscillator): `x' = y`, `y' = -x`. Origin is a center.
pub fn center() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields = vec![(id("x"), sym("y")), (id("y"), neg(sym("x")))];
    (fields, xy())
}

/// Saddle: `x' = x`, `y' = -y`. Origin is a saddle.
pub fn saddle() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields = vec![(id("x"), sym("x")), (id("y"), neg(sym("y")))];
    (fields, xy())
}

/// Damped oscillator: `x' = y`, `y' = -x - 0.3 y`. Origin is a stable spiral.
pub fn damped_oscillator() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields = vec![
        (id("x"), sym("y")),
        (id("y"), Expr::difference(neg(sym("x")), Expr::product(Expr::constant(0.3), sym("y")))),
    ];
    (fields, xy())
}

/// Unstable spiral: `x' = y`, `y' = -x + 0.3 y`. Origin is an unstable spiral.
pub fn unstable_spiral() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields = vec![
        (id("x"), sym("y")),
        (id("y"), Expr::sum(neg(sym("x")), Expr::product(Expr::constant(0.3), sym("y")))),
    ];
    (fields, xy())
}

/// Undamped pendulum: `x' = y`, `y' = -sin(x)`. A center at (0,0) and saddles at
/// (±π, 0).
pub fn pendulum() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields = vec![(id("x"), sym("y")), (id("y"), neg(sin(sym("x"))))];
    (fields, xy())
}

/// Lotka–Volterra (`a=b=c=d=1`): `x' = x - x y`, `y' = -y + x y`. A saddle at
/// (0,0) and a center at (1,1).
pub fn lotka_volterra() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields = vec![
        (id("x"), Expr::difference(sym("x"), Expr::product(sym("x"), sym("y")))),
        (id("y"), Expr::sum(neg(sym("y")), Expr::product(sym("x"), sym("y")))),
    ];
    (fields, xy())
}

/// Transcritical normal form (single state): `x' = x^2`. A marginal
/// (non-hyperbolic) fixed point at the origin.
pub fn transcritical() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields =
        vec![(id("x"), Expr::binary(BinaryOperator::Power, sym("x"), Expr::constant(2.0)))];
    (fields, vec![id("x")])
}

/// No fixed point (single state): `x' = 2 + cos(x)`, which never vanishes.
pub fn no_fixed_point() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields =
        vec![(id("x"), Expr::sum(Expr::constant(2.0), Expr::unary(UnaryOperator::Cos, sym("x"))))];
    (fields, vec![id("x")])
}

/// Approximate coordinate equality for asserting located roots.
pub fn close(a: &[f64], b: &[f64], tolerance: f64) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() <= tolerance)
}
