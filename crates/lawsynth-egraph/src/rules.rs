use lawsynth_expr::{BinaryOperator, Expr};

/// Local rules intentionally restricted to algebraic identities valid for all
/// real-valued operands in the expression language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteRule {
    Simplify,
    CanonicalCommutativeOrder,
}

/// Applies safe local simplification and canonical ordering for addition and
/// multiplication, then reports the normalized expression.
pub fn normalize(expression: Expr) -> Expr {
    normalize_inner(expression).simplify()
}

fn normalize_inner(expression: Expr) -> Expr {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => expression,
        Expr::Unary { operator, operand } => Expr::unary(operator, normalize_inner(*operand)),
        Expr::Binary {
            operator,
            left,
            right,
        } => {
            let left = normalize_inner(*left);
            let right = normalize_inner(*right);
            if matches!(operator, BinaryOperator::Add | BinaryOperator::Multiply)
                && right.to_canonical_string() < left.to_canonical_string()
            {
                Expr::binary(operator, right, left)
            } else {
                Expr::binary(operator, left, right)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_core::Identifier;
    #[test]
    fn canonicalizes_commutative_operand_order() {
        let x = Expr::symbol(Identifier::new("x").unwrap());
        let y = Expr::symbol(Identifier::new("y").unwrap());
        assert_eq!(
            normalize(Expr::sum(y.clone(), x.clone())),
            normalize(Expr::sum(x, y))
        );
    }
}
