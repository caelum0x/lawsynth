//! Shared fixtures for the basin-mapping integration tests.
//!
//! Each builder returns `(fields, states)` for a small autonomous vector field
//! with a known basin structure, so the tests can assert the mapping against the
//! analytic answer.

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};

/// Convenience constructor for a valid identifier.
pub fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// `x^3` as an expression.
fn cube(symbol: &Identifier) -> Expr {
    Expr::binary(BinaryOperator::Power, Expr::symbol(symbol.clone()), Expr::constant(3.0))
}

/// Bistable 1-D flow `ẋ = x − x³`.
///
/// Stable attractors at `x = ±1`, an (unstable) saddle at `x = 0`.
pub fn bistable() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let x = id("x");
    let field = Expr::difference(Expr::symbol(x.clone()), cube(&x));
    (vec![(x.clone(), field)], vec![x])
}

/// Damped, unforced Duffing oscillator `ẋ = y, ẏ = x − x³ − 0.5 y`.
///
/// Two stable spirals at `(±1, 0)` and a saddle at the origin — a genuine
/// double-well with inertia.
pub fn duffing() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let x = id("x");
    let y = id("y");
    let fx = Expr::symbol(y.clone());
    let fy = Expr::difference(
        Expr::difference(Expr::symbol(x.clone()), cube(&x)),
        Expr::product(Expr::constant(0.5), Expr::symbol(y.clone())),
    );
    (vec![(x.clone(), fx), (y.clone(), fy)], vec![x, y])
}

/// Damped linear oscillator `ẋ = y, ẏ = −x − 0.5 y`.
///
/// A single global attractor: a stable spiral at the origin.
pub fn damped_oscillator() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let x = id("x");
    let y = id("y");
    let fx = Expr::symbol(y.clone());
    let fy = Expr::difference(
        Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone())),
        Expr::product(Expr::constant(0.5), Expr::symbol(y.clone())),
    );
    (vec![(x.clone(), fx), (y.clone(), fy)], vec![x, y])
}

/// Divergent linear flow `ẋ = x`.
///
/// The only fixed point (the origin) is an unstable node, so there is no
/// attractor; every non-origin initial condition runs off to infinity.
pub fn divergent() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let x = id("x");
    (vec![(x.clone(), Expr::symbol(x.clone()))], vec![x])
}

/// Pure saddle `ẋ = x, ẏ = −y`.
///
/// The origin is a saddle: no stable attractor exists anywhere in the box.
pub fn pure_saddle() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let x = id("x");
    let y = id("y");
    let fx = Expr::symbol(x.clone());
    let fy = Expr::unary(UnaryOperator::Negate, Expr::symbol(y.clone()));
    (vec![(x.clone(), fx), (y.clone(), fy)], vec![x, y])
}
