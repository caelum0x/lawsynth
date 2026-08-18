//! Shared fixtures for the Lyapunov-spectrum integration tests.
//!
//! The helpers build small autonomous vector fields as `lawsynth-expr` fields:
//! linear decay, harmonic and damped oscillators, and the Lorenz system. Each is
//! a case with a known spectrum or a known divergence (trace) that the estimator
//! is checked against.

use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator};

/// Convenience constructor for a valid identifier.
pub fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// A fully specified test model: fields and their state ordering.
pub struct Model {
    pub fields: Vec<(Identifier, Expr)>,
    pub states: Vec<Identifier>,
}

fn neg(expr: Expr) -> Expr {
    Expr::unary(UnaryOperator::Negate, expr)
}

/// One-dimensional linear decay `ẋ = −x`. Exact spectrum `{−1}`.
pub fn linear_decay_1d() -> Model {
    let x = id("x");
    let field = neg(Expr::symbol(x.clone()));
    Model { fields: vec![(x.clone(), field)], states: vec![x] }
}

/// Diagonal linear decay `ẋ = −x`, `ẏ = −2y`. Exact spectrum `{−1, −2}`.
pub fn linear_decay_2d() -> Model {
    let x = id("x");
    let y = id("y");
    let fx = neg(Expr::symbol(x.clone()));
    let fy = neg(Expr::product(Expr::constant(2.0), Expr::symbol(y.clone())));
    Model { fields: vec![(x.clone(), fx), (y.clone(), fy)], states: vec![x, y] }
}

/// Undamped harmonic oscillator `ẋ = y`, `ẏ = −x`. Conservative: both exponents
/// are zero (trace of `J` is zero, no separation).
pub fn harmonic_oscillator() -> Model {
    let x = id("x");
    let y = id("y");
    let fx = Expr::symbol(y.clone());
    let fy = neg(Expr::symbol(x.clone()));
    Model { fields: vec![(x.clone(), fx), (y.clone(), fy)], states: vec![x, y] }
}

/// Damped harmonic oscillator `ẋ = y`, `ẏ = −x − 0.3y`. Both exponents are
/// negative and sum to the constant trace `−0.3`.
pub fn damped_oscillator() -> Model {
    let x = id("x");
    let y = id("y");
    let fx = Expr::symbol(y.clone());
    // -x - 0.3 y
    let fy = Expr::difference(
        neg(Expr::symbol(x.clone())),
        Expr::product(Expr::constant(0.3), Expr::symbol(y.clone())),
    );
    Model { fields: vec![(x.clone(), fx), (y.clone(), fy)], states: vec![x, y] }
}

/// The Lorenz system `ẋ = σ(y−x)`, `ẏ = x(ρ−z)−y`, `ż = xy−βz` with the classic
/// chaotic parameters `σ = 10`, `ρ = 28`, `β = 8/3`. Constant divergence
/// `tr J = −(σ + 1 + β) = −(41/3) ≈ −13.6667`.
pub fn lorenz() -> Model {
    let x = id("x");
    let y = id("y");
    let z = id("z");

    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    // σ (y − x)
    let fx = Expr::product(
        Expr::constant(sigma),
        Expr::difference(Expr::symbol(y.clone()), Expr::symbol(x.clone())),
    );
    // x (ρ − z) − y
    let fy = Expr::difference(
        Expr::product(
            Expr::symbol(x.clone()),
            Expr::difference(Expr::constant(rho), Expr::symbol(z.clone())),
        ),
        Expr::symbol(y.clone()),
    );
    // x y − β z
    let fz = Expr::difference(
        Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone())),
        Expr::product(Expr::constant(beta), Expr::symbol(z.clone())),
    );

    Model { fields: vec![(x.clone(), fx), (y.clone(), fy), (z.clone(), fz)], states: vec![x, y, z] }
}

/// The exact Lorenz divergence `tr J = −(σ + 1 + β)` for `σ = 10`, `β = 8/3`.
pub fn lorenz_divergence() -> f64 {
    -(10.0 + 1.0 + 8.0 / 3.0)
}
