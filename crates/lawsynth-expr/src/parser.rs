use std::fmt;

use lawsynth_core::Identifier;

use crate::{BinaryOperator, Expr, UnaryOperator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    position: usize,
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.position)
    }
}

impl std::error::Error for ParseError {}

/// Parses scalar arithmetic with parentheses and elementary scalar functions.
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(input)?;
    let mut parser = Parser { tokens, index: 0 };
    let expression = parser.sum()?;
    if !matches!(parser.peek(), Token::End) {
        return Err(parser.error("unexpected token"));
    }
    Ok(expression)
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Symbol(String),
    Operator(u8),
    LeftParen,
    RightParen,
    End,
}

fn tokenize(input: &str) -> Result<Vec<(usize, Token)>, ParseError> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_whitespace() => index += 1,
            b'+' | b'-' | b'*' | b'/' | b'^' => {
                tokens.push((index, Token::Operator(bytes[index])));
                index += 1;
            }
            b'(' => {
                tokens.push((index, Token::LeftParen));
                index += 1;
            }
            b')' => {
                tokens.push((index, Token::RightParen));
                index += 1;
            }
            byte if byte.is_ascii_digit() || byte == b'.' => {
                let start = index;
                while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.')
                {
                    index += 1;
                }
                if index < bytes.len() && matches!(bytes[index], b'e' | b'E') {
                    index += 1;
                    if index < bytes.len() && matches!(bytes[index], b'+' | b'-') {
                        index += 1;
                    }
                    let exponent_start = index;
                    while index < bytes.len() && bytes[index].is_ascii_digit() {
                        index += 1;
                    }
                    if index == exponent_start {
                        return Err(ParseError {
                            position: start,
                            message: "invalid exponent".to_owned(),
                        });
                    }
                }
                let value = input[start..index].parse().map_err(|_| ParseError {
                    position: start,
                    message: "invalid number".to_owned(),
                })?;
                tokens.push((start, Token::Number(value)));
            }
            byte if byte.is_ascii_alphabetic() || byte == b'_' => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
                {
                    index += 1;
                }
                tokens.push((start, Token::Symbol(input[start..index].to_owned())));
            }
            _ => {
                return Err(ParseError {
                    position: index,
                    message: "invalid character".to_owned(),
                });
            }
        }
    }
    tokens.push((input.len(), Token::End));
    Ok(tokens)
}

struct Parser {
    tokens: Vec<(usize, Token)>,
    index: usize,
}

impl Parser {
    fn sum(&mut self) -> Result<Expr, ParseError> {
        let mut expression = self.product()?;
        while let Token::Operator(operator @ (b'+' | b'-')) = self.peek() {
            let operator = *operator;
            self.advance();
            let right = self.product()?;
            expression = if operator == b'+' {
                Expr::sum(expression, right)
            } else {
                Expr::difference(expression, right)
            };
        }
        Ok(expression)
    }

    fn product(&mut self) -> Result<Expr, ParseError> {
        let mut expression = self.power()?;
        while let Token::Operator(operator @ (b'*' | b'/')) = self.peek() {
            let operator = *operator;
            self.advance();
            let right = self.power()?;
            expression = if operator == b'*' {
                Expr::product(expression, right)
            } else {
                Expr::quotient(expression, right)
            };
        }
        Ok(expression)
    }

    fn power(&mut self) -> Result<Expr, ParseError> {
        let left = self.unary()?;
        if matches!(self.peek(), Token::Operator(b'^')) {
            self.advance();
            Ok(Expr::binary(BinaryOperator::Power, left, self.power()?))
        } else {
            Ok(left)
        }
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Token::Operator(b'-')) {
            self.advance();
            Ok(Expr::unary(UnaryOperator::Negate, self.unary()?))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance().1 {
            Token::Number(value) if value.is_finite() => Ok(Expr::constant(value)),
            Token::Number(_) => Err(self.error("number must be finite")),
            Token::Symbol(name) if matches!(self.peek(), Token::LeftParen) => {
                self.advance();
                let argument = self.sum()?;
                if !matches!(self.advance().1, Token::RightParen) {
                    return Err(self.error("expected closing parenthesis"));
                }
                match name.as_str() {
                    "exp" => Ok(Expr::unary(UnaryOperator::Exp, argument)),
                    "log" => Ok(Expr::unary(UnaryOperator::Log, argument)),
                    "sin" => Ok(Expr::unary(UnaryOperator::Sin, argument)),
                    "cos" => Ok(Expr::unary(UnaryOperator::Cos, argument)),
                    _ => Err(self.error("unknown function")),
                }
            }
            Token::Symbol(name) => {
                Identifier::new(name).map(Expr::symbol).map_err(|_| self.error("invalid symbol"))
            }
            Token::LeftParen => {
                let expression = self.sum()?;
                if matches!(self.advance().1, Token::RightParen) {
                    Ok(expression)
                } else {
                    Err(self.error("expected closing parenthesis"))
                }
            }
            _ => Err(self.error("expected expression")),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.index].1
    }

    fn advance(&mut self) -> (usize, Token) {
        let token = self.tokens[self.index].clone();
        if !matches!(token.1, Token::End) {
            self.index += 1;
        }
        token
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError { position: self.tokens[self.index].0, message: message.to_owned() }
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use crate::{Environment, evaluate};

    use super::parse;

    #[test]
    fn respects_precedence_and_functions() {
        let expression = parse("2 * x + log(exp(1))").unwrap();
        let value =
            evaluate(&expression, &Environment::from([(Identifier::new("x").unwrap(), 3.0)]))
                .unwrap();
        assert!((value - 7.0).abs() < 1e-12);
    }

    #[test]
    fn parses_trigonometric_functions() {
        let expression = parse("sin(0) + cos(0)").unwrap();
        assert_eq!(evaluate(&expression, &Environment::new()).unwrap(), 1.0);
    }
}
