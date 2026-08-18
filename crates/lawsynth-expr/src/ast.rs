use lawsynth_core::{Identifier, stable_hash};

/// Operators with one argument in the scalar expression language.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Exp,
    Log,
    Sin,
    Cos,
}

/// Operators with two arguments in the scalar expression language.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}

/// The initial scalar subset of World IR expressions.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Constant(f64),
    Symbol(Identifier),
    Unary { operator: UnaryOperator, operand: Box<Expr> },
    Binary { operator: BinaryOperator, left: Box<Expr>, right: Box<Expr> },
}

impl Expr {
    pub fn constant(value: f64) -> Self {
        Self::Constant(value)
    }

    pub fn symbol(identifier: Identifier) -> Self {
        Self::Symbol(identifier)
    }

    pub fn unary(operator: UnaryOperator, operand: Expr) -> Self {
        Self::Unary { operator, operand: Box::new(operand) }
    }

    pub fn binary(operator: BinaryOperator, left: Expr, right: Expr) -> Self {
        Self::Binary { operator, left: Box::new(left), right: Box::new(right) }
    }

    pub fn sum(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOperator::Add, left, right)
    }

    pub fn difference(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOperator::Subtract, left, right)
    }

    pub fn product(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOperator::Multiply, left, right)
    }

    pub fn quotient(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOperator::Divide, left, right)
    }

    /// Returns the symbolic derivative with respect to one scalar symbol.
    pub fn derivative(&self, symbol: &Identifier) -> Self {
        match self {
            Self::Constant(_) => Self::constant(0.0),
            Self::Symbol(id) => Self::constant(if id == symbol { 1.0 } else { 0.0 }),
            Self::Unary { operator, operand } => match operator {
                UnaryOperator::Negate => {
                    Self::unary(UnaryOperator::Negate, operand.derivative(symbol))
                }
                UnaryOperator::Exp => Self::product(
                    Self::unary(UnaryOperator::Exp, (**operand).clone()),
                    operand.derivative(symbol),
                ),
                UnaryOperator::Log => {
                    Self::quotient(operand.derivative(symbol), (**operand).clone())
                }
                UnaryOperator::Sin => Self::product(
                    Self::unary(UnaryOperator::Cos, (**operand).clone()),
                    operand.derivative(symbol),
                ),
                UnaryOperator::Cos => Self::unary(
                    UnaryOperator::Negate,
                    Self::product(
                        Self::unary(UnaryOperator::Sin, (**operand).clone()),
                        operand.derivative(symbol),
                    ),
                ),
            },
            Self::Binary { operator, left, right } => match operator {
                BinaryOperator::Add => Self::sum(left.derivative(symbol), right.derivative(symbol)),
                BinaryOperator::Subtract => {
                    Self::difference(left.derivative(symbol), right.derivative(symbol))
                }
                BinaryOperator::Multiply => Self::sum(
                    Self::product(left.derivative(symbol), (**right).clone()),
                    Self::product((**left).clone(), right.derivative(symbol)),
                ),
                BinaryOperator::Divide => Self::quotient(
                    Self::difference(
                        Self::product(left.derivative(symbol), (**right).clone()),
                        Self::product((**left).clone(), right.derivative(symbol)),
                    ),
                    Self::product((**right).clone(), (**right).clone()),
                ),
                BinaryOperator::Power => Self::product(
                    Self::binary(BinaryOperator::Power, (**left).clone(), (**right).clone()),
                    Self::sum(
                        Self::product(
                            right.derivative(symbol),
                            Self::unary(UnaryOperator::Log, (**left).clone()),
                        ),
                        Self::product(
                            (**right).clone(),
                            Self::quotient(left.derivative(symbol), (**left).clone()),
                        ),
                    ),
                ),
            },
        }
    }

    /// Applies local algebraic and constant-folding reductions deterministically.
    pub fn simplify(&self) -> Self {
        match self {
            Self::Constant(_) | Self::Symbol(_) => self.clone(),
            Self::Unary { operator, operand } => {
                let operand = operand.simplify();
                match (operator, &operand) {
                    (UnaryOperator::Negate, Self::Constant(value)) => Self::constant(-value),
                    (
                        UnaryOperator::Negate,
                        Self::Unary { operator: UnaryOperator::Negate, operand },
                    ) => (**operand).clone(),
                    (UnaryOperator::Exp, Self::Constant(value)) if value.exp().is_finite() => {
                        Self::constant(value.exp())
                    }
                    (UnaryOperator::Log, Self::Constant(value)) if *value > 0.0 => {
                        Self::constant(value.ln())
                    }
                    (UnaryOperator::Sin, Self::Constant(value)) => Self::constant(value.sin()),
                    (UnaryOperator::Cos, Self::Constant(value)) => Self::constant(value.cos()),
                    _ => Self::unary(*operator, operand),
                }
            }
            Self::Binary { operator, left, right } => {
                let left = left.simplify();
                let right = right.simplify();
                if let (Self::Constant(left_value), Self::Constant(right_value)) = (&left, &right) {
                    let value = match operator {
                        BinaryOperator::Add => Some(left_value + right_value),
                        BinaryOperator::Subtract => Some(left_value - right_value),
                        BinaryOperator::Multiply => Some(left_value * right_value),
                        BinaryOperator::Divide if *right_value != 0.0 => {
                            Some(left_value / right_value)
                        }
                        BinaryOperator::Power => Some(left_value.powf(*right_value)),
                        BinaryOperator::Divide => None,
                    };
                    if let Some(value) = value.filter(|value| value.is_finite()) {
                        return Self::constant(value);
                    }
                }
                match (operator, &left, &right) {
                    (BinaryOperator::Add, Self::Constant(0.0), _) => right,
                    (BinaryOperator::Add, _, Self::Constant(0.0)) => left,
                    (BinaryOperator::Subtract, _, Self::Constant(0.0)) => left,
                    (BinaryOperator::Multiply, Self::Constant(0.0), _)
                    | (BinaryOperator::Multiply, _, Self::Constant(0.0)) => Self::constant(0.0),
                    (BinaryOperator::Multiply, Self::Constant(1.0), _) => right,
                    (BinaryOperator::Multiply, _, Self::Constant(1.0)) => left,
                    (BinaryOperator::Divide, Self::Constant(0.0), _) => Self::constant(0.0),
                    (BinaryOperator::Divide, _, Self::Constant(1.0)) => left,
                    (BinaryOperator::Power, _, Self::Constant(0.0)) => Self::constant(1.0),
                    (BinaryOperator::Power, _, Self::Constant(1.0)) => left,
                    _ => Self::binary(*operator, left, right),
                }
            }
        }
    }

    /// A stable structural fingerprint. It is suitable for deterministic IR
    /// ordering, but not for cryptographic verification.
    pub fn fingerprint(&self) -> u64 {
        stable_hash(self.to_canonical_string())
    }

    pub fn to_canonical_string(&self) -> String {
        match self {
            Self::Constant(value) => format!("constant:{value:.17e}"),
            Self::Symbol(identifier) => format!("symbol:{}", identifier.as_str()),
            Self::Unary { operator, operand } => {
                format!("unary:{operator:?}({})", operand.to_canonical_string())
            }
            Self::Binary { operator, left, right } => format!(
                "binary:{operator:?}({},{})",
                left.to_canonical_string(),
                right.to_canonical_string()
            ),
        }
    }
}
