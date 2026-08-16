use crate::BinaryOperator;

/// Parser precedence, where larger values bind more tightly.
pub const fn binary_precedence(operator: BinaryOperator) -> u8 {
    match operator {
        BinaryOperator::Add | BinaryOperator::Subtract => 1,
        BinaryOperator::Multiply | BinaryOperator::Divide => 2,
        BinaryOperator::Power => 3,
    }
}

/// Whether the binary operation can be reordered without changing real arithmetic semantics.
pub const fn is_commutative(operator: BinaryOperator) -> bool {
    matches!(operator, BinaryOperator::Add | BinaryOperator::Multiply)
}
