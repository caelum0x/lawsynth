use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator, is_commutative};

/// The categories of local rewrite this engine applies. Every rule is a
/// value-preserving identity: it never changes what an expression evaluates to
/// at any point where the *original* expression is defined. Rules whose
/// rewritten form is defined on a *wider* domain than the original (for example
/// `x / x -> 1`, which also has a value at `x = 0`) are still value-preserving
/// on the original's domain; those domain caveats are documented per rule below
/// and in `specs/egraph-simplification/README.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteRule {
    /// Local algebraic reduction and constant folding (`x+0`, `x*1`, `x^0`,
    /// `log(exp(x))`, `sin(-x)`, distributive factoring, …).
    Simplify,
    /// Deterministic canonical ordering of commutative operands.
    CanonicalCommutativeOrder,
}

/// Upper bound on normalization sweeps. Each sweep that changes the expression
/// strictly reduces its node count or re-sorts commutative operands into their
/// (idempotent) canonical order, so a fixpoint is always reached well within
/// this bound; the cap only guarantees termination on unexpected input.
const MAX_NORMALIZE_SWEEPS: usize = 256;

/// Applies safe local simplification, constant folding, and canonical ordering
/// to a local fixpoint, then returns the normalized expression. The result is
/// deterministic and idempotent: `normalize(normalize(e)) == normalize(e)`.
pub fn normalize(expression: Expr) -> Expr {
    let mut current = expression;
    for _ in 0..MAX_NORMALIZE_SWEEPS {
        let next = rewrite(&current);
        if next == current {
            break;
        }
        current = next;
    }
    current
}

/// One bottom-up rewrite sweep: children are rewritten first, then the local
/// rule set is applied to the resulting node.
fn rewrite(expression: &Expr) -> Expr {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => expression.clone(),
        Expr::Unary { operator, operand } => reduce_unary(*operator, rewrite(operand)),
        Expr::Binary { operator, left, right } => {
            reduce_binary(*operator, rewrite(left), rewrite(right))
        }
    }
}

/// Exact equality of an `f64` to a target, written so that only comparisons
/// against the literal `0.0` reach the float comparison (which is exact for the
/// small integer-valued targets we test here). `-0.0` compares equal to `0.0`.
fn equals(value: f64, target: f64) -> bool {
    (value - target) == 0.0
}

fn is_constant(expression: &Expr, target: f64) -> bool {
    matches!(expression, Expr::Constant(value) if equals(*value, target))
}

/// Folds a binary operation over two constants, returning `None` when the
/// operation is undefined (division by zero) or non-finite (overflow, `0^-1`).
fn fold(operator: BinaryOperator, left: f64, right: f64) -> Option<f64> {
    let value = match operator {
        BinaryOperator::Add => left + right,
        BinaryOperator::Subtract => left - right,
        BinaryOperator::Multiply => left * right,
        BinaryOperator::Divide if equals(right, 0.0) => return None,
        BinaryOperator::Divide => left / right,
        BinaryOperator::Power => left.powf(right),
    };
    value.is_finite().then_some(value)
}

fn reduce_unary(operator: UnaryOperator, operand: Expr) -> Expr {
    match operator {
        // -c -> constant; -(-x) -> x. Both are exact and unconditional.
        UnaryOperator::Negate => match operand {
            Expr::Constant(value) => Expr::constant(-value),
            Expr::Unary { operator: UnaryOperator::Negate, operand: inner } => *inner,
            other => Expr::unary(UnaryOperator::Negate, other),
        },
        // exp(c) folds when finite; exp(log(x)) -> x is exact for x > 0 (the
        // only domain on which log(x) is defined), so it is value-preserving
        // wherever the original is defined.
        UnaryOperator::Exp => match &operand {
            Expr::Constant(value) if value.exp().is_finite() => Expr::constant(value.exp()),
            Expr::Unary { operator: UnaryOperator::Log, operand: inner } => (**inner).clone(),
            _ => Expr::unary(UnaryOperator::Exp, operand),
        },
        // log(c) folds for c > 0; log(exp(x)) -> x is exact everywhere because
        // exp(x) > 0 for all real x.
        UnaryOperator::Log => match &operand {
            Expr::Constant(value) if *value > 0.0 => Expr::constant(value.ln()),
            Expr::Unary { operator: UnaryOperator::Exp, operand: inner } => (**inner).clone(),
            _ => Expr::unary(UnaryOperator::Log, operand),
        },
        // sin(-x) -> -sin(x): sine is odd, so this holds for all real x.
        UnaryOperator::Sin => match operand {
            Expr::Constant(value) => Expr::constant(value.sin()),
            Expr::Unary { operator: UnaryOperator::Negate, operand: inner } => {
                Expr::unary(UnaryOperator::Negate, Expr::unary(UnaryOperator::Sin, *inner))
            }
            other => Expr::unary(UnaryOperator::Sin, other),
        },
        // cos(-x) -> cos(x): cosine is even, so this holds for all real x.
        UnaryOperator::Cos => match operand {
            Expr::Constant(value) => Expr::constant(value.cos()),
            Expr::Unary { operator: UnaryOperator::Negate, operand: inner } => {
                Expr::unary(UnaryOperator::Cos, *inner)
            }
            other => Expr::unary(UnaryOperator::Cos, other),
        },
    }
}

fn reduce_binary(operator: BinaryOperator, left: Expr, right: Expr) -> Expr {
    // Canonical ordering for commutative operators keeps `x + y` and `y + x`
    // structurally identical. The order is a total, deterministic function of
    // the operands, so applying it twice is a no-op.
    let (left, right) =
        if is_commutative(operator) && right.to_canonical_string() < left.to_canonical_string() {
            (right, left)
        } else {
            (left, right)
        };

    if let (Expr::Constant(left_value), Expr::Constant(right_value)) = (&left, &right) {
        if let Some(value) = fold(operator, *left_value, *right_value) {
            return Expr::constant(value);
        }
    }

    match operator {
        BinaryOperator::Add => reduce_add(left, right),
        BinaryOperator::Subtract => reduce_subtract(left, right),
        BinaryOperator::Multiply => reduce_multiply(left, right),
        BinaryOperator::Divide => reduce_divide(left, right),
        BinaryOperator::Power => reduce_power(left, right),
    }
}

fn reduce_add(left: Expr, right: Expr) -> Expr {
    // x + 0 -> x (both orders; after canonical ordering a constant sorts first).
    if is_constant(&left, 0.0) {
        return right;
    }
    if is_constant(&right, 0.0) {
        return left;
    }
    // sin(u)^2 + cos(u)^2 -> 1 (the Pythagorean identity, exact for all real u).
    if let Some(one) = pythagorean_identity(&left, &right) {
        return one;
    }
    // a*b + a*c -> a*(b + c): distributive factoring, the cost-reducing
    // direction of distributivity. Value-preserving for all reals.
    if let Some(factored) = factor_sum(&left, &right) {
        return factored;
    }
    Expr::sum(left, right)
}

fn reduce_subtract(left: Expr, right: Expr) -> Expr {
    if is_constant(&right, 0.0) {
        return left; // x - 0 -> x
    }
    if is_constant(&left, 0.0) {
        return Expr::unary(UnaryOperator::Negate, right); // 0 - x -> -x
    }
    if left == right {
        return Expr::constant(0.0); // x - x -> 0 (value-preserving where x is defined)
    }
    Expr::difference(left, right)
}

fn reduce_multiply(left: Expr, right: Expr) -> Expr {
    // x * 0 -> 0. Value-preserving wherever x is defined; where x is undefined
    // the original is undefined too, so no new incorrect value is produced.
    if is_constant(&left, 0.0) || is_constant(&right, 0.0) {
        return Expr::constant(0.0);
    }
    if is_constant(&left, 1.0) {
        return right; // 1 * x -> x
    }
    if is_constant(&right, 1.0) {
        return left; // x * 1 -> x
    }
    // exp(a) * exp(b) -> exp(a + b), exact for all real a, b.
    if let (
        Expr::Unary { operator: UnaryOperator::Exp, operand: left_argument },
        Expr::Unary { operator: UnaryOperator::Exp, operand: right_argument },
    ) = (&left, &right)
    {
        return Expr::unary(
            UnaryOperator::Exp,
            Expr::sum((**left_argument).clone(), (**right_argument).clone()),
        );
    }
    // x^a * x^b -> x^(a + b) for a shared base (exact for base > 0).
    if let Some(combined) = combine_power_product(&left, &right) {
        return combined;
    }
    Expr::product(left, right)
}

fn reduce_divide(left: Expr, right: Expr) -> Expr {
    if is_constant(&right, 1.0) {
        return left; // x / 1 -> x
    }
    // 0 / x -> 0 for x != 0 (guarded so `0 / 0` is never rewritten to 0).
    if is_constant(&left, 0.0) && !matches!(right, Expr::Constant(_)) {
        return Expr::constant(0.0);
    }
    // x / x -> 1 for x != 0. Guarded against the `0 / 0` literal case.
    if left == right && !is_constant(&left, 0.0) {
        return Expr::constant(1.0);
    }
    Expr::quotient(left, right)
}

fn reduce_power(left: Expr, right: Expr) -> Expr {
    if is_constant(&right, 0.0) {
        return Expr::constant(1.0); // x^0 -> 1 (consistent with powf, including 0^0 = 1)
    }
    if is_constant(&right, 1.0) {
        return left; // x^1 -> x
    }
    // (x^a)^b -> x^(a*b) for a shared base (exact for base > 0).
    if let Expr::Binary { operator: BinaryOperator::Power, left: base, right: inner_exponent } =
        &left
    {
        return Expr::binary(
            BinaryOperator::Power,
            (**base).clone(),
            Expr::product((**inner_exponent).clone(), right),
        );
    }
    Expr::binary(BinaryOperator::Power, left, right)
}

/// Matches `f(u)^2` where `f` is `sin` or `cos`, returning `(f, u)`.
fn squared_trigonometric(expression: &Expr) -> Option<(UnaryOperator, &Expr)> {
    if let Expr::Binary { operator: BinaryOperator::Power, left, right } = expression {
        if is_constant(right, 2.0) {
            if let Expr::Unary { operator, operand } = left.as_ref() {
                if matches!(operator, UnaryOperator::Sin | UnaryOperator::Cos) {
                    return Some((*operator, operand.as_ref()));
                }
            }
        }
    }
    None
}

/// `sin(u)^2 + cos(u)^2 -> 1` (in either operand order, same `u`).
fn pythagorean_identity(left: &Expr, right: &Expr) -> Option<Expr> {
    let (left_op, left_arg) = squared_trigonometric(left)?;
    let (right_op, right_arg) = squared_trigonometric(right)?;
    let complementary = matches!(
        (left_op, right_op),
        (UnaryOperator::Sin, UnaryOperator::Cos) | (UnaryOperator::Cos, UnaryOperator::Sin)
    );
    (complementary && left_arg == right_arg).then(|| Expr::constant(1.0))
}

/// `x^a * x^b -> x^(a + b)` when both factors are powers of a shared base.
fn combine_power_product(left: &Expr, right: &Expr) -> Option<Expr> {
    let (left_base, left_exponent) = as_power(left)?;
    let (right_base, right_exponent) = as_power(right)?;
    (left_base == right_base).then(|| {
        Expr::binary(
            BinaryOperator::Power,
            left_base.clone(),
            Expr::sum(left_exponent.clone(), right_exponent.clone()),
        )
    })
}

fn as_power(expression: &Expr) -> Option<(&Expr, &Expr)> {
    if let Expr::Binary { operator: BinaryOperator::Power, left, right } = expression {
        Some((left.as_ref(), right.as_ref()))
    } else {
        None
    }
}

/// `a*b + a*c -> a*(b + c)`, factoring out a common multiplicand shared by the
/// two products in any position. Strictly reduces node count.
fn factor_sum(left: &Expr, right: &Expr) -> Option<Expr> {
    let (la, lb) = as_product(left)?;
    let (ra, rb) = as_product(right)?;
    let (common, rest_left, rest_right) = if la == ra {
        (la, lb, rb)
    } else if la == rb {
        (la, lb, ra)
    } else if lb == ra {
        (lb, la, rb)
    } else if lb == rb {
        (lb, la, ra)
    } else {
        return None;
    };
    Some(Expr::product(common.clone(), Expr::sum(rest_left.clone(), rest_right.clone())))
}

fn as_product(expression: &Expr) -> Option<(&Expr, &Expr)> {
    if let Expr::Binary { operator: BinaryOperator::Multiply, left, right } = expression {
        Some((left.as_ref(), right.as_ref()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_core::Identifier;

    fn symbol(name: &str) -> Expr {
        Expr::symbol(Identifier::new(name).unwrap())
    }

    #[test]
    fn canonicalizes_commutative_operand_order() {
        let x = symbol("x");
        let y = symbol("y");
        assert_eq!(normalize(Expr::sum(y.clone(), x.clone())), normalize(Expr::sum(x, y)));
    }

    #[test]
    fn folds_nested_constants() {
        assert_eq!(
            normalize(Expr::product(Expr::constant(2.0), Expr::constant(3.0))),
            Expr::constant(6.0)
        );
        assert_eq!(
            normalize(Expr::sum(Expr::constant(1.0), Expr::constant(1.0))),
            Expr::constant(2.0)
        );
    }

    #[test]
    fn removes_additive_and_multiplicative_identities() {
        let x = symbol("x");
        assert_eq!(normalize(Expr::sum(x.clone(), Expr::constant(0.0))), x);
        assert_eq!(normalize(Expr::difference(x.clone(), Expr::constant(0.0))), x);
        assert_eq!(normalize(Expr::product(x.clone(), Expr::constant(1.0))), x);
        assert_eq!(normalize(Expr::product(x.clone(), Expr::constant(0.0))), Expr::constant(0.0));
        assert_eq!(normalize(Expr::quotient(x.clone(), Expr::constant(1.0))), x);
    }

    #[test]
    fn cancels_subtraction_and_division_of_equal_terms() {
        let x = symbol("x");
        assert_eq!(normalize(Expr::difference(x.clone(), x.clone())), Expr::constant(0.0));
        assert_eq!(normalize(Expr::quotient(x.clone(), x.clone())), Expr::constant(1.0));
    }

    #[test]
    fn rewrites_zero_minus_x_to_negation() {
        let x = symbol("x");
        assert_eq!(
            normalize(Expr::difference(Expr::constant(0.0), x.clone())),
            Expr::unary(UnaryOperator::Negate, x)
        );
    }

    #[test]
    fn applies_power_identities() {
        let x = symbol("x");
        assert_eq!(
            normalize(Expr::binary(BinaryOperator::Power, x.clone(), Expr::constant(1.0))),
            x.clone()
        );
        assert_eq!(
            normalize(Expr::binary(BinaryOperator::Power, x, Expr::constant(0.0))),
            Expr::constant(1.0)
        );
    }

    #[test]
    fn combines_powers_with_shared_base() {
        let x = symbol("x");
        let left = Expr::binary(BinaryOperator::Power, x.clone(), Expr::constant(2.0));
        let right = Expr::binary(BinaryOperator::Power, x.clone(), Expr::constant(3.0));
        assert_eq!(
            normalize(Expr::product(left, right)),
            Expr::binary(BinaryOperator::Power, x, Expr::constant(5.0))
        );
    }

    #[test]
    fn collapses_nested_powers() {
        let x = symbol("x");
        let inner = Expr::binary(BinaryOperator::Power, x.clone(), Expr::constant(2.0));
        let outer = Expr::binary(BinaryOperator::Power, inner, Expr::constant(3.0));
        assert_eq!(normalize(outer), Expr::binary(BinaryOperator::Power, x, Expr::constant(6.0)));
    }

    #[test]
    fn cancels_log_exp_inverses() {
        let x = symbol("x");
        assert_eq!(
            normalize(Expr::unary(UnaryOperator::Log, Expr::unary(UnaryOperator::Exp, x.clone()))),
            x.clone()
        );
        assert_eq!(
            normalize(Expr::unary(UnaryOperator::Exp, Expr::unary(UnaryOperator::Log, x.clone()))),
            x
        );
    }

    #[test]
    fn applies_trigonometric_parity() {
        let x = symbol("x");
        assert_eq!(
            normalize(Expr::unary(
                UnaryOperator::Cos,
                Expr::unary(UnaryOperator::Negate, x.clone())
            )),
            Expr::unary(UnaryOperator::Cos, x.clone())
        );
        assert_eq!(
            normalize(Expr::unary(
                UnaryOperator::Sin,
                Expr::unary(UnaryOperator::Negate, x.clone())
            )),
            Expr::unary(UnaryOperator::Negate, Expr::unary(UnaryOperator::Sin, x))
        );
    }

    #[test]
    fn collapses_pythagorean_identity() {
        let x = symbol("x");
        let sine = Expr::binary(
            BinaryOperator::Power,
            Expr::unary(UnaryOperator::Sin, x.clone()),
            Expr::constant(2.0),
        );
        let cosine = Expr::binary(
            BinaryOperator::Power,
            Expr::unary(UnaryOperator::Cos, x),
            Expr::constant(2.0),
        );
        assert_eq!(normalize(Expr::sum(sine, cosine)), Expr::constant(1.0));
    }

    #[test]
    fn factors_common_multiplicand() {
        let a = symbol("a");
        let b = symbol("b");
        let c = symbol("c");
        let factored = normalize(Expr::sum(
            Expr::product(a.clone(), b.clone()),
            Expr::product(a.clone(), c.clone()),
        ));
        // a*b + a*c has 7 nodes; a*(b + c) has 5.
        assert_eq!(crate::expression_cost(&factored), 5);
    }

    #[test]
    fn combines_exponential_product() {
        let a = symbol("a");
        let b = symbol("b");
        let product = Expr::product(
            Expr::unary(UnaryOperator::Exp, a.clone()),
            Expr::unary(UnaryOperator::Exp, b.clone()),
        );
        let normalized = normalize(product);
        // Result is exp(a + b) (with a + b in canonical order); it has 4 nodes.
        assert_eq!(crate::expression_cost(&normalized), 4);
        assert!(matches!(normalized, Expr::Unary { operator: UnaryOperator::Exp, .. }));
    }

    #[test]
    fn double_negation_cancels() {
        let x = symbol("x");
        assert_eq!(
            normalize(Expr::unary(
                UnaryOperator::Negate,
                Expr::unary(UnaryOperator::Negate, x.clone())
            )),
            x
        );
    }

    #[test]
    fn normalization_reaches_a_fixpoint() {
        let x = symbol("x");
        let once = normalize(Expr::product(
            Expr::sum(Expr::constant(0.0), x.clone()),
            Expr::constant(1.0),
        ));
        assert_eq!(normalize(once.clone()), once);
        assert_eq!(once, x);
    }
}
