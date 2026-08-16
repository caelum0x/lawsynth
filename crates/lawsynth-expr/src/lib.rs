//! Language-neutral, deterministic scalar expression IR.

mod ast;
mod config;
mod error;
mod evaluate;
mod literal;
mod node;
mod operator;
mod parser;
mod printer;
mod symbol;

pub use ast::{BinaryOperator, Expr, UnaryOperator};
pub use config::ExpressionConfig;
pub use error::EvaluationError;
pub use evaluate::{Environment, evaluate};
pub use literal::Literal;
pub use node::ExpressionNode;
pub use operator::{binary_precedence, is_commutative};
pub use parser::{ParseError, parse};
pub use printer::print;
pub use symbol::symbols;
