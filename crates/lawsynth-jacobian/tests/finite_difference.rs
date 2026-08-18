//! Cross-checks the analytic (symbolic) Jacobian against independent numerical
//! finite differences produced by `lawsynth-differentiate`. This is the key
//! honesty test: two unrelated code paths — symbolic differentiation here and
//! the three-point Lagrange derivative there — must agree at sampled points.

use lawsynth_core::Identifier;
use lawsynth_differentiate::differentiate_series;
use lawsynth_expr::{BinaryOperator, Environment, Expr, UnaryOperator, evaluate};
use lawsynth_jacobian::{Jacobian, analytic_jacobian};

const STEP: f64 = 1e-4;
const TOLERANCE: f64 = 1e-6;

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

/// Numerically estimates `∂field/∂wrt` at `point` using a symmetric three-point
/// grid fed through `lawsynth-differentiate`'s series derivative. The middle
/// sample of that estimator is exactly the central difference, so this is a
/// genuinely independent check of the symbolic result.
fn finite_partial(field: &Expr, wrt: &Identifier, point: &Environment) -> f64 {
    let center = *point.get(wrt).expect("point must define the differentiation symbol");
    let times = [center - STEP, center, center + STEP];
    let values: Vec<f64> = times
        .iter()
        .map(|&value| {
            let mut environment = point.clone();
            environment.insert(wrt.clone(), value);
            evaluate(field, &environment).expect("field must evaluate on the probe grid")
        })
        .collect();
    let derivative = differentiate_series(&times, &values).expect("series derivative");
    derivative[1]
}

/// Asserts the symbolic Jacobian matches finite differences of `fields` at every
/// point in `points`, entry by entry.
fn assert_matches_finite_differences(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    points: &[Environment],
) {
    let jacobian: Jacobian = analytic_jacobian(fields, states).unwrap();
    for point in points {
        let symbolic = jacobian.evaluate(point).unwrap();
        for (row_index, row_state) in states.iter().enumerate() {
            let field = &fields.iter().find(|(target, _)| target == row_state).unwrap().1;
            for (col_index, col_state) in states.iter().enumerate() {
                let numeric = finite_partial(field, col_state, point);
                let analytic = symbolic[row_index][col_index];
                assert!(
                    (analytic - numeric).abs() <= TOLERANCE,
                    "mismatch at J[{row_index}][{col_index}] (d{row_state}/d{col_state}): \
                     symbolic={analytic}, finite={numeric}"
                );
            }
        }
    }
}

fn point(pairs: &[(&str, f64)]) -> Environment {
    pairs.iter().map(|(name, value)| (id(name), *value)).collect()
}

#[test]
fn lotka_volterra_2x2_matches_finite_differences() {
    // x' = 1.5 x - x y ; y' = -3 y + x y
    let x = id("x");
    let y = id("y");
    let fields = vec![
        (
            x.clone(),
            Expr::difference(
                Expr::product(Expr::constant(1.5), Expr::symbol(x.clone())),
                Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone())),
            ),
        ),
        (
            y.clone(),
            Expr::sum(
                Expr::product(Expr::constant(-3.0), Expr::symbol(y.clone())),
                Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone())),
            ),
        ),
    ];
    let states = vec![x, y];
    let points = [
        point(&[("x", 1.0), ("y", 1.0)]),
        point(&[("x", 2.5), ("y", 0.4)]),
        point(&[("x", 0.3), ("y", 3.2)]),
        point(&[("x", -1.2), ("y", 1.7)]),
    ];
    assert_matches_finite_differences(&fields, &states, &points);
}

#[test]
fn lorenz_3x3_matches_finite_differences() {
    // Lorenz system: x' = 10(y - x), y' = x(28 - z) - y, z' = x y - (8/3) z
    let x = id("x");
    let y = id("y");
    let z = id("z");
    let fields = vec![
        (
            x.clone(),
            Expr::product(
                Expr::constant(10.0),
                Expr::difference(Expr::symbol(y.clone()), Expr::symbol(x.clone())),
            ),
        ),
        (
            y.clone(),
            Expr::difference(
                Expr::product(
                    Expr::symbol(x.clone()),
                    Expr::difference(Expr::constant(28.0), Expr::symbol(z.clone())),
                ),
                Expr::symbol(y.clone()),
            ),
        ),
        (
            z.clone(),
            Expr::difference(
                Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone())),
                Expr::product(Expr::constant(8.0 / 3.0), Expr::symbol(z.clone())),
            ),
        ),
    ];
    let states = vec![x, y, z];
    let points = [
        point(&[("x", 1.0), ("y", 1.0), ("z", 1.0)]),
        point(&[("x", -6.0), ("y", 2.0), ("z", 20.0)]),
        point(&[("x", 8.0), ("y", -3.5), ("z", 27.0)]),
    ];
    assert_matches_finite_differences(&fields, &states, &points);
}

#[test]
fn transcendental_field_matches_finite_differences() {
    // Exercises chain/quotient/exp/log/sin rules through the numeric check.
    // x' = sin(x * y) + exp(-y)
    // y' = log(x^2 + 1) - x / y
    let x = id("x");
    let y = id("y");
    let fields = vec![
        (
            x.clone(),
            Expr::sum(
                Expr::unary(
                    UnaryOperator::Sin,
                    Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone())),
                ),
                Expr::unary(
                    UnaryOperator::Exp,
                    Expr::unary(UnaryOperator::Negate, Expr::symbol(y.clone())),
                ),
            ),
        ),
        (
            y.clone(),
            Expr::difference(
                Expr::unary(
                    UnaryOperator::Log,
                    Expr::sum(
                        Expr::binary(
                            BinaryOperator::Power,
                            Expr::symbol(x.clone()),
                            Expr::constant(2.0),
                        ),
                        Expr::constant(1.0),
                    ),
                ),
                Expr::quotient(Expr::symbol(x.clone()), Expr::symbol(y.clone())),
            ),
        ),
    ];
    let states = vec![x, y];
    let points = [
        point(&[("x", 0.6), ("y", 1.3)]),
        point(&[("x", -0.9), ("y", 2.1)]),
        point(&[("x", 1.4), ("y", 0.8)]),
    ];
    assert_matches_finite_differences(&fields, &states, &points);
}

#[test]
fn identical_inputs_are_bit_identical() {
    // Determinism down to structure and float bits: two independent builds of
    // the same field must agree exactly.
    let x = id("x");
    let y = id("y");
    let build = || {
        let fields = vec![
            (
                x.clone(),
                Expr::difference(
                    Expr::product(Expr::constant(1.5), Expr::symbol(x.clone())),
                    Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone())),
                ),
            ),
            (
                y.clone(),
                Expr::product(
                    Expr::symbol(x.clone()),
                    Expr::binary(
                        BinaryOperator::Power,
                        Expr::symbol(y.clone()),
                        Expr::constant(3.0),
                    ),
                ),
            ),
        ];
        analytic_jacobian(&fields, &[x.clone(), y.clone()]).unwrap()
    };
    let first = build();
    let second = build();

    // Structural equality.
    assert_eq!(first.to_canonical_string(), second.to_canonical_string());

    // Numeric equality down to the last bit.
    let environment = point(&[("x", 1.234), ("y", -0.777)]);
    let left = first.evaluate(&environment).unwrap();
    let right = second.evaluate(&environment).unwrap();
    for (left_row, right_row) in left.iter().zip(right.iter()) {
        for (left_value, right_value) in left_row.iter().zip(right_row.iter()) {
            assert_eq!(left_value.to_bits(), right_value.to_bits());
        }
    }
}
