//! Shared fixtures for the MPC integration tests: canonical controlled plants
//! (double integrator, forced pendulum, forced Van der Pol, a generic linear
//! system) plus a reference RK4 rollout for building ground-truth baselines.
#![allow(dead_code)]

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};
use lawsynth_koopman::Matrix;

/// Convenience: an identifier that is known to be valid.
pub fn id(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// A single field built from a state/control linear combination
/// `Σ a_j x_j + Σ b_k u_k`, folded left-to-right from a zero seed.
fn linear_row(
    state_coeffs: &[f64],
    states: &[Identifier],
    control_coeffs: &[f64],
    controls: &[Identifier],
) -> Expr {
    let mut expr = Expr::constant(0.0);
    for (coeff, symbol) in state_coeffs.iter().zip(states) {
        expr = Expr::sum(expr, Expr::product(Expr::constant(*coeff), Expr::symbol(symbol.clone())));
    }
    for (coeff, symbol) in control_coeffs.iter().zip(controls) {
        expr = Expr::sum(expr, Expr::product(Expr::constant(*coeff), Expr::symbol(symbol.clone())));
    }
    expr
}

/// A 2-state, 1-control model of the field bundle plus its symbols.
pub struct Plant {
    pub fields: Vec<(Identifier, Expr)>,
    pub states: Vec<Identifier>,
    pub controls: Vec<Identifier>,
}

/// Double integrator `ẋ = y, ẏ = u`. Linear and controllable.
pub fn double_integrator() -> Plant {
    let (x, y, u) = (id("x"), id("y"), id("u"));
    let fields = vec![(x.clone(), Expr::symbol(y.clone())), (y.clone(), Expr::symbol(u.clone()))];
    Plant { fields, states: vec![x, y], controls: vec![u] }
}

/// Forced pendulum `ẋ = y, ẏ = −sin(x) + u`. Nonlinear, equilibrium at origin.
pub fn forced_pendulum() -> Plant {
    let (x, y, u) = (id("x"), id("y"), id("u"));
    let dy = Expr::sum(
        Expr::unary(
            UnaryOperator::Negate,
            Expr::unary(UnaryOperator::Sin, Expr::symbol(x.clone())),
        ),
        Expr::symbol(u.clone()),
    );
    let fields = vec![(x.clone(), Expr::symbol(y.clone())), (y.clone(), dy)];
    Plant { fields, states: vec![x, y], controls: vec![u] }
}

/// Forced Van der Pol `ẋ = y, ẏ = μ(1 − x²)y − x + u`.
///
/// With `μ > 0` and no control the origin is an unstable focus and trajectories
/// spiral out to a limit cycle; with control the pair is stabilizable.
pub fn forced_van_der_pol(mu: f64) -> Plant {
    let (x, y, u) = (id("x"), id("y"), id("u"));
    let x_squared =
        Expr::binary(BinaryOperator::Power, Expr::symbol(x.clone()), Expr::constant(2.0));
    let damping = Expr::product(
        Expr::product(Expr::constant(mu), Expr::difference(Expr::constant(1.0), x_squared)),
        Expr::symbol(y.clone()),
    );
    let dy = Expr::sum(Expr::difference(damping, Expr::symbol(x.clone())), Expr::symbol(u.clone()));
    let fields = vec![(x.clone(), Expr::symbol(y.clone())), (y.clone(), dy)];
    Plant { fields, states: vec![x, y], controls: vec![u] }
}

/// A generic 2-state, 1-control linear system `ẋ = A x + B u` built from raw
/// coefficient arrays, returned alongside the exact dense `A` and `B` matrices
/// so a test can compare successive-linearization output against them.
pub fn linear_system(a: [[f64; 2]; 2], b: [[f64; 1]; 2]) -> (Plant, Matrix, Matrix) {
    let (x, y, u) = (id("x"), id("y"), id("u"));
    let states = vec![x.clone(), y.clone()];
    let controls = vec![u.clone()];
    let fields = vec![
        (x.clone(), linear_row(&a[0], &states, &b[0], &controls)),
        (y.clone(), linear_row(&a[1], &states, &b[1], &controls)),
    ];
    let a_matrix = Matrix::from_rows(vec![a[0].to_vec(), a[1].to_vec()]).unwrap();
    let b_matrix = Matrix::from_rows(vec![b[0].to_vec(), b[1].to_vec()]).unwrap();
    (Plant { fields, states, controls }, a_matrix, b_matrix)
}

/// The `n × n` identity, as a convenience for building `Q`.
pub fn identity(n: usize) -> Matrix {
    Matrix::identity(n)
}

/// A 1×1 control weight `R = [r]`.
pub fn scalar_weight(r: f64) -> Matrix {
    Matrix::from_rows(vec![vec![r]]).unwrap()
}

/// Classical fixed-step RK4 rollout of `ẋ = deriv(x)` (control folded into the
/// closure), returning `steps + 1` states including the initial one. This is the
/// independent ground truth the MPC results are checked against.
pub fn rk4_rollout<F>(deriv: F, x0: Vec<f64>, dt: f64, steps: usize) -> Vec<Vec<f64>>
where
    F: Fn(&[f64]) -> Vec<f64>,
{
    let mut state = x0;
    let mut trajectory = vec![state.clone()];
    for _ in 0..steps {
        let k1 = deriv(&state);
        let k2 = deriv(&axpy(&state, &k1, dt / 2.0));
        let k3 = deriv(&axpy(&state, &k2, dt / 2.0));
        let k4 = deriv(&axpy(&state, &k3, dt));
        state = state
            .iter()
            .enumerate()
            .map(|(i, value)| value + (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]))
            .collect();
        trajectory.push(state.clone());
    }
    trajectory
}

/// Returns `base + scale · delta`, element-wise (a shared RK4 stage helper).
pub fn axpy(base: &[f64], delta: &[f64], scale: f64) -> Vec<f64> {
    base.iter().zip(delta).map(|(value, slope)| value + scale * slope).collect()
}

/// Euclidean norm of a vector.
pub fn norm(vector: &[f64]) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}
