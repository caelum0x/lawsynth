//! Shared fixtures: textbook parameterized vector fields with known bifurcations.

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr};

/// Convenience: an identifier that is known to be valid.
pub fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// `x^n` with a constant exponent.
fn power(base: Expr, exponent: f64) -> Expr {
    Expr::binary(BinaryOperator::Power, base, Expr::constant(exponent))
}

/// Saddle-node normal form: `x' = mu - x^2`.
///
/// No real fixed point for `mu < 0`; two (`x = ±√mu`) for `mu > 0`; they collide
/// at `mu = 0`.
pub fn saddle_node() -> (Vec<(Identifier, Expr)>, Vec<Identifier>, Identifier) {
    let x = id("x");
    let mu = id("mu");
    let field = Expr::difference(Expr::symbol(mu.clone()), power(Expr::symbol(x.clone()), 2.0));
    (vec![(x.clone(), field)], vec![x], mu)
}

/// Transcritical normal form: `x' = mu*x - x^2`.
///
/// Fixed points `x = 0` and `x = mu` for all `mu`; they exchange stability at
/// `mu = 0`.
pub fn transcritical() -> (Vec<(Identifier, Expr)>, Vec<Identifier>, Identifier) {
    let x = id("x");
    let mu = id("mu");
    let field = Expr::difference(
        Expr::product(Expr::symbol(mu.clone()), Expr::symbol(x.clone())),
        power(Expr::symbol(x.clone()), 2.0),
    );
    (vec![(x.clone(), field)], vec![x], mu)
}

/// Supercritical pitchfork normal form: `x' = mu*x - x^3`.
///
/// One fixed point (`x = 0`) for `mu < 0`; three (`x = 0, ±√mu`) for `mu > 0`.
pub fn pitchfork() -> (Vec<(Identifier, Expr)>, Vec<Identifier>, Identifier) {
    let x = id("x");
    let mu = id("mu");
    let field = Expr::difference(
        Expr::product(Expr::symbol(mu.clone()), Expr::symbol(x.clone())),
        power(Expr::symbol(x.clone()), 3.0),
    );
    (vec![(x.clone(), field)], vec![x], mu)
}

/// Hopf normal form (2D):
/// `x' = mu*x - y - x*(x^2 + y^2)`, `y' = x + mu*y - y*(x^2 + y^2)`.
///
/// The origin is a fixed point for all `mu`, with eigenvalues `mu ± i`: a
/// complex pair crosses the imaginary axis at `mu = 0`.
pub fn hopf() -> (Vec<(Identifier, Expr)>, Vec<Identifier>, Identifier) {
    let x = id("x");
    let y = id("y");
    let mu = id("mu");
    // r^2 = x^2 + y^2
    let radius_sq =
        Expr::sum(power(Expr::symbol(x.clone()), 2.0), power(Expr::symbol(y.clone()), 2.0));
    // x' = mu*x - y - x*r^2
    let dx = Expr::difference(
        Expr::difference(
            Expr::product(Expr::symbol(mu.clone()), Expr::symbol(x.clone())),
            Expr::symbol(y.clone()),
        ),
        Expr::product(Expr::symbol(x.clone()), radius_sq.clone()),
    );
    // y' = x + mu*y - y*r^2
    let dy = Expr::difference(
        Expr::sum(
            Expr::symbol(x.clone()),
            Expr::product(Expr::symbol(mu.clone()), Expr::symbol(y.clone())),
        ),
        Expr::product(Expr::symbol(y.clone()), radius_sq),
    );
    (vec![(x.clone(), dx), (y.clone(), dy)], vec![x, y], mu)
}
