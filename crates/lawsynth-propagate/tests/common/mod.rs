//! Shared fixtures for the forecast-uncertainty integration tests.
//!
//! The helpers build small discovered models as `lawsynth-expr` fields, mirroring
//! the sensitivity crate's fixtures, plus a couple of closed-form references used
//! to anchor the delta method against analytic truth.

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};

/// Convenience constructor for a valid identifier.
pub fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// A fully specified test model: fields, states, and parameters.
pub struct Model {
    pub fields: Vec<(Identifier, Expr)>,
    pub states: Vec<Identifier>,
    pub parameters: Vec<Identifier>,
}

/// Linear decay `ẋ = −θ·x`, one state, one parameter.
///
/// Closed form: `x(t) = x0·e^{−θt}`, `∂x/∂θ = −t·x0·e^{−θt}`.
pub fn linear_decay() -> Model {
    let x = id("x");
    let theta = id("theta");
    let field = Expr::product(
        Expr::unary(UnaryOperator::Negate, Expr::symbol(theta.clone())),
        Expr::symbol(x.clone()),
    );
    Model { fields: vec![(x.clone(), field)], states: vec![x], parameters: vec![theta] }
}

/// Linear growth `ẋ = θ·x`, one state, one parameter.
///
/// Closed form: `x(t) = x0·e^{θt}`, `∂x/∂θ = t·x0·e^{θt}` — a sensitivity (and so
/// a delta variance) that is strictly increasing in `t`.
pub fn linear_growth() -> Model {
    let x = id("x");
    let theta = id("theta");
    let field = Expr::product(Expr::symbol(theta.clone()), Expr::symbol(x.clone()));
    Model { fields: vec![(x.clone(), field)], states: vec![x], parameters: vec![theta] }
}

/// Scalar logistic growth `ẋ = θ1·x − θ2·x²`, one state, two parameters.
pub fn logistic() -> Model {
    let x = id("x");
    let theta1 = id("theta1");
    let theta2 = id("theta2");
    let growth = Expr::product(Expr::symbol(theta1.clone()), Expr::symbol(x.clone()));
    let crowding = Expr::product(
        Expr::symbol(theta2.clone()),
        Expr::binary(BinaryOperator::Power, Expr::symbol(x.clone()), Expr::constant(2.0)),
    );
    let field = Expr::difference(growth, crowding);
    Model { fields: vec![(x.clone(), field)], states: vec![x], parameters: vec![theta1, theta2] }
}

/// Quadratic growth `ẋ = p1·x + p2·x²`, one state, two parameters.
///
/// The parameters map sign-for-sign onto a `[x, x²]` regression, so a bootstrap
/// ensemble over that library feeds this model directly (with `p2 < 0` it is a
/// saturating, logistic-shaped law).
pub fn poly_growth() -> Model {
    let x = id("x");
    let p1 = id("p1");
    let p2 = id("p2");
    let linear = Expr::product(Expr::symbol(p1.clone()), Expr::symbol(x.clone()));
    let quadratic = Expr::product(
        Expr::symbol(p2.clone()),
        Expr::binary(BinaryOperator::Power, Expr::symbol(x.clone()), Expr::constant(2.0)),
    );
    let field = Expr::sum(linear, quadratic);
    Model { fields: vec![(x.clone(), field)], states: vec![x], parameters: vec![p1, p2] }
}
