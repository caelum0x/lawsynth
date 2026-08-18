use crate::WasmError;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Constant(f64),
    Variable(String),
    Neg(Box<Self>),
    Binary { op: BinaryOp, left: Box<Self>, right: Box<Self> },
    Function { name: Function, argument: Box<Self> },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Function {
    Sin,
    Cos,
    Exp,
    Log,
    Sqrt,
    Abs,
}

impl Expression {
    pub fn parse(source: &str) -> Result<Self, WasmError> {
        let mut parser = Parser::new(tokenize(source)?);
        let expression = parser.sum()?;
        if parser.peek().is_some() {
            return Err(WasmError::InvalidExpression("unexpected trailing token".into()));
        }
        Ok(expression)
    }
    pub fn evaluate(&self, values: &BTreeMap<String, f64>) -> Result<f64, WasmError> {
        let result = match self {
            Self::Constant(value) => *value,
            Self::Variable(name) => *values
                .get(name)
                .ok_or_else(|| WasmError::InvalidExpression(format!("unknown variable {name}")))?,
            Self::Neg(value) => -value.evaluate(values)?,
            Self::Binary { op, left, right } => {
                let a = left.evaluate(values)?;
                let b = right.evaluate(values)?;
                match op {
                    BinaryOp::Add => a + b,
                    BinaryOp::Subtract => a - b,
                    BinaryOp::Multiply => a * b,
                    BinaryOp::Divide => {
                        if b == 0.0 {
                            return Err(WasmError::Simulation("division by zero".into()));
                        }
                        a / b
                    }
                    BinaryOp::Power => a.powf(b),
                }
            }
            Self::Function { name, argument } => {
                let value = argument.evaluate(values)?;
                match name {
                    Function::Sin => value.sin(),
                    Function::Cos => value.cos(),
                    Function::Exp => value.exp(),
                    Function::Log => {
                        if value <= 0.0 {
                            return Err(WasmError::Simulation(
                                "logarithm requires a positive value".into(),
                            ));
                        }
                        value.ln()
                    }
                    Function::Sqrt => {
                        if value < 0.0 {
                            return Err(WasmError::Simulation(
                                "square root requires a nonnegative value".into(),
                            ));
                        }
                        value.sqrt()
                    }
                    Function::Abs => value.abs(),
                }
            }
        };
        if result.is_finite() {
            Ok(result)
        } else {
            Err(WasmError::Simulation("expression evaluated to a non-finite value".into()))
        }
    }
    pub fn source(&self) -> String {
        match self {
            Self::Constant(value) => value.to_string(),
            Self::Variable(name) => name.clone(),
            Self::Neg(value) => format!("-({})", value.source()),
            Self::Binary { op, left, right } => {
                let glyph = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Subtract => "-",
                    BinaryOp::Multiply => "*",
                    BinaryOp::Divide => "/",
                    BinaryOp::Power => "^",
                };
                format!("({}{}{})", left.source(), glyph, right.source())
            }
            Self::Function { name, argument } => {
                let label = match name {
                    Function::Sin => "sin",
                    Function::Cos => "cos",
                    Function::Exp => "exp",
                    Function::Log => "log",
                    Function::Sqrt => "sqrt",
                    Function::Abs => "abs",
                };
                format!("{label}({})", argument.source())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Name(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LeftParen,
    RightParen,
}

fn tokenize(source: &str) -> Result<Vec<Token>, WasmError> {
    let chars: Vec<char> = source.chars().collect();
    let mut at = 0;
    let mut tokens = Vec::new();
    while at < chars.len() {
        let c = chars[at];
        if c.is_whitespace() {
            at += 1;
            continue;
        }
        let token = match c {
            '+' => {
                at += 1;
                Token::Plus
            }
            '-' => {
                at += 1;
                Token::Minus
            }
            '*' => {
                at += 1;
                Token::Star
            }
            '/' => {
                at += 1;
                Token::Slash
            }
            '^' => {
                at += 1;
                Token::Caret
            }
            '(' => {
                at += 1;
                Token::LeftParen
            }
            ')' => {
                at += 1;
                Token::RightParen
            }
            '0'..='9' | '.' => {
                let start = at;
                at += 1;
                while at < chars.len() && (chars[at].is_ascii_digit() || chars[at] == '.') {
                    at += 1;
                }
                if at < chars.len() && matches!(chars[at], 'e' | 'E') {
                    at += 1;
                    if at < chars.len() && matches!(chars[at], '+' | '-') {
                        at += 1;
                    }
                    let exponent = at;
                    while at < chars.len() && chars[at].is_ascii_digit() {
                        at += 1;
                    }
                    if exponent == at {
                        return Err(WasmError::InvalidExpression("invalid exponent".into()));
                    }
                }
                let number: String = chars[start..at].iter().collect();
                let value: f64 = number.parse().map_err(|_| {
                    WasmError::InvalidExpression(format!("invalid number {number}"))
                })?;
                if !value.is_finite() {
                    return Err(WasmError::InvalidExpression("number must be finite".into()));
                }
                Token::Number(value)
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = at;
                at += 1;
                while at < chars.len() && (chars[at].is_ascii_alphanumeric() || chars[at] == '_') {
                    at += 1;
                }
                Token::Name(chars[start..at].iter().collect())
            }
            _ => {
                return Err(WasmError::InvalidExpression(format!("unsupported character {c}")));
            }
        };
        tokens.push(token);
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}
impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, at: 0 }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }
    fn take(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.at).cloned();
        if token.is_some() {
            self.at += 1;
        }
        token
    }
    fn sum(&mut self) -> Result<Expression, WasmError> {
        let mut left = self.product()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinaryOp::Add,
                Some(Token::Minus) => BinaryOp::Subtract,
                _ => return Ok(left),
            };
            self.take();
            left =
                Expression::Binary { op, left: Box::new(left), right: Box::new(self.product()?) };
        }
    }
    fn product(&mut self) -> Result<Expression, WasmError> {
        let mut left = self.power()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinaryOp::Multiply,
                Some(Token::Slash) => BinaryOp::Divide,
                _ => return Ok(left),
            };
            self.take();
            left = Expression::Binary { op, left: Box::new(left), right: Box::new(self.power()?) };
        }
    }
    fn power(&mut self) -> Result<Expression, WasmError> {
        let left = self.unary()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.take();
            Ok(Expression::Binary {
                op: BinaryOp::Power,
                left: Box::new(left),
                right: Box::new(self.power()?),
            })
        } else {
            Ok(left)
        }
    }
    fn unary(&mut self) -> Result<Expression, WasmError> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.take();
            Ok(Expression::Neg(Box::new(self.unary()?)))
        } else {
            self.primary()
        }
    }
    fn primary(&mut self) -> Result<Expression, WasmError> {
        match self.take() {
            Some(Token::Number(value)) => Ok(Expression::Constant(value)),
            Some(Token::Name(name)) if matches!(self.peek(), Some(Token::LeftParen)) => {
                self.take();
                let argument = self.sum()?;
                if !matches!(self.take(), Some(Token::RightParen)) {
                    return Err(WasmError::InvalidExpression("unclosed function call".into()));
                }
                let function = match name.as_str() {
                    "sin" => Function::Sin,
                    "cos" => Function::Cos,
                    "exp" => Function::Exp,
                    "log" => Function::Log,
                    "sqrt" => Function::Sqrt,
                    "abs" => Function::Abs,
                    _ => {
                        return Err(WasmError::InvalidExpression(format!(
                            "unknown function {name}"
                        )));
                    }
                };
                Ok(Expression::Function { name: function, argument: Box::new(argument) })
            }
            Some(Token::Name(name)) => Ok(Expression::Variable(name)),
            Some(Token::LeftParen) => {
                let expression = self.sum()?;
                if !matches!(self.take(), Some(Token::RightParen)) {
                    return Err(WasmError::InvalidExpression("unclosed parenthesis".into()));
                }
                Ok(expression)
            }
            _ => Err(WasmError::InvalidExpression("expected an expression".into())),
        }
    }
}
