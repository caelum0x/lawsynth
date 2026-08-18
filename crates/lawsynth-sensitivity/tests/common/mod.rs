//! Shared fixtures for the forward-sensitivity integration tests.
//!
//! The helpers build small discovered models as `lawsynth-expr` fields and
//! provide a state-only re-simulator used to cross-check the analytic
//! sensitivities against a central finite difference in parameter space.

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Environment, Expr, evaluate};

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

/// Scalar logistic growth `ẋ = θ1·x − θ2·x²`, one state, two parameters.
pub fn logistic() -> Model {
    let x = id("x");
    let theta1 = id("theta1");
    let theta2 = id("theta2");
    // theta1 * x - theta2 * x^2
    let growth = Expr::product(Expr::symbol(theta1.clone()), Expr::symbol(x.clone()));
    let crowding = Expr::product(
        Expr::symbol(theta2.clone()),
        Expr::binary(BinaryOperator::Power, Expr::symbol(x.clone()), Expr::constant(2.0)),
    );
    let field = Expr::difference(growth, crowding);
    Model { fields: vec![(x.clone(), field)], states: vec![x], parameters: vec![theta1, theta2] }
}

/// Two-species Lotka–Volterra with four parameters:
/// `ẋ = a·x − b·x·y`, `ẏ = d·x·y − c·y`.
pub fn lotka_volterra() -> Model {
    let x = id("x");
    let y = id("y");
    let a = id("a");
    let b = id("b");
    let c = id("c");
    let d = id("d");

    // a*x - b*x*y
    let prey = Expr::difference(
        Expr::product(Expr::symbol(a.clone()), Expr::symbol(x.clone())),
        Expr::product(
            Expr::product(Expr::symbol(b.clone()), Expr::symbol(x.clone())),
            Expr::symbol(y.clone()),
        ),
    );
    // d*x*y - c*y
    let predator = Expr::difference(
        Expr::product(
            Expr::product(Expr::symbol(d.clone()), Expr::symbol(x.clone())),
            Expr::symbol(y.clone()),
        ),
        Expr::product(Expr::symbol(c.clone()), Expr::symbol(y.clone())),
    );

    Model {
        fields: vec![(x.clone(), prey), (y.clone(), predator)],
        states: vec![x, y],
        parameters: vec![a, b, c, d],
    }
}

/// Integrates the STATE ONLY with the same fixed-step RK4 scheme, at the given
/// parameter values. Used to build finite-difference references without touching
/// the sensitivity machinery under test.
pub fn simulate_state(
    model: &Model,
    initial: &[f64],
    parameter_values: &[f64],
    dt: f64,
    steps: usize,
) -> Vec<f64> {
    let ordered_fields: Vec<&Expr> = model
        .states
        .iter()
        .map(|state| &model.fields.iter().find(|(target, _)| target == state).unwrap().1)
        .collect();

    let mut state = initial.to_vec();
    for _ in 0..steps {
        let k1 = eval_field(model, &ordered_fields, &state, parameter_values);
        let k2 = eval_field(model, &ordered_fields, &axpy(&state, dt / 2.0, &k1), parameter_values);
        let k3 = eval_field(model, &ordered_fields, &axpy(&state, dt / 2.0, &k2), parameter_values);
        let k4 = eval_field(model, &ordered_fields, &axpy(&state, dt, &k3), parameter_values);
        for i in 0..state.len() {
            state[i] += (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
    }
    state
}

fn eval_field(
    model: &Model,
    ordered_fields: &[&Expr],
    state: &[f64],
    parameter_values: &[f64],
) -> Vec<f64> {
    let mut environment = Environment::new();
    for (parameter, value) in model.parameters.iter().zip(parameter_values) {
        environment.insert(parameter.clone(), *value);
    }
    for (identifier, value) in model.states.iter().zip(state) {
        environment.insert(identifier.clone(), *value);
    }
    ordered_fields.iter().map(|field| evaluate(field, &environment).unwrap()).collect()
}

fn axpy(base: &[f64], scale: f64, direction: &[f64]) -> Vec<f64> {
    base.iter().zip(direction).map(|(b, d)| b + scale * d).collect()
}

/// Central finite-difference sensitivity `∂x(t_final)/∂θ_j` obtained by
/// re-simulating the state at `θ ± h·e_j`.
pub fn finite_difference_sensitivity(
    model: &Model,
    initial: &[f64],
    parameter_values: &[f64],
    parameter_index: usize,
    step_size: f64,
    dt: f64,
    steps: usize,
) -> Vec<f64> {
    let mut plus = parameter_values.to_vec();
    let mut minus = parameter_values.to_vec();
    plus[parameter_index] += step_size;
    minus[parameter_index] -= step_size;

    let forward = simulate_state(model, initial, &plus, dt, steps);
    let backward = simulate_state(model, initial, &minus, dt, steps);

    forward.iter().zip(&backward).map(|(f, b)| (f - b) / (2.0 * step_size)).collect()
}
