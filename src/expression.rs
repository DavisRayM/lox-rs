use std::error::Error;

use crate::{
    errors::RuntimeError,
    token::{self, Literal, Token},
    token_type::TokenType,
};

pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Group(Box<Expression>),
    Literal(token::Literal),
    Unary(Token, Box<Expression>),
}

impl Expression {
    pub fn eval(&self) -> Result<Literal, RuntimeError> {
        match self {
            Self::Literal(literal) => Ok(literal.to_owned()),
            Self::Group(expr) => expr.eval(),
            Self::Unary(op, right) => {
                let right = right.eval()?;
                match op.token_type {
                    TokenType::Minus => {
                        if let Literal::Number(num) = right {
                            Ok(Literal::Number(-num))
                        } else {
                            Err(RuntimeError {
                                cause: "'-' can only be used on numerical values.".to_string(),
                            })
                        }
                    }
                    TokenType::Bang => {
                        if let Literal::Boolean(b) = right {
                            Ok(Literal::Boolean(!b))
                        } else {
                            Err(RuntimeError {
                                cause: "! operator can only be used on boolean values.".to_string(),
                            })
                        }
                    }
                    _ => Err(RuntimeError {
                        cause: format!("unexpected operator {:?}", op.token_type),
                    }),
                }
            }
            Self::Binary(left, op, right) => {
                let left = left.eval()?;
                let right = right.eval()?;

                if let Literal::Number(left) = left {
                    if let Literal::Number(right) = right {
                        match op.token_type {
                            TokenType::Minus => return Ok(Literal::Number(left - right)),
                            TokenType::Slash => return Ok(Literal::Number(left / right)),
                            TokenType::Star => return Ok(Literal::Number(left * right)),
                            TokenType::Plus => return Ok(Literal::Number(left + right)),
                            TokenType::Greater => return Ok(Literal::Boolean(left > right)),
                            TokenType::GreaterEqual => return Ok(Literal::Boolean(left >= right)),
                            TokenType::Less => return Ok(Literal::Boolean(left < right)),
                            TokenType::LessEqual => return Ok(Literal::Boolean(left <= right)),
                            _ => {
                                return Err(RuntimeError {
                                    cause: format!("unexpected operator {:?}", op.token_type),
                                })
                            }
                        }
                    }
                }

                match op.token_type {
                    TokenType::BangEqual => Ok(Literal::Boolean(left != right)),
                    TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
                    _ => Err(RuntimeError {
                        cause: "invalid expression".to_string(),
                    }),
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn display_text(&self) -> String {
        match self {
            Self::Group(expr) => {
                format!("(group {})", expr.display_text())
            }
            Self::Literal(lit) => match lit {
                Literal::Number(val) => format!("{}", val),
                Literal::Boolean(val) => format!("{}", val),
                Literal::String(val) => val.iter().collect::<String>(),
                Literal::None => "".into(),
            },
            Self::Unary(op, right) => {
                format!("({} {})", op.lexeme, right.display_text())
            }
            Self::Binary(left, op, right) => {
                format!(
                    "({} {} {})",
                    op.lexeme,
                    left.display_text(),
                    right.display_text()
                )
            }
        }
    }
}

pub struct ExpressionBuilder {
    left: Option<Expression>,
    right: Option<Expression>,
    group: Option<Expression>,
    op: Option<Token>,
    literal: Option<token::Literal>,
}

impl ExpressionBuilder {
    pub fn new() -> Self {
        ExpressionBuilder {
            left: None,
            right: None,
            op: None,
            literal: None,
            group: None,
        }
    }

    pub fn left_expression(mut self, expr: Expression) -> Self {
        self.left = Some(expr);
        self
    }

    pub fn right_expression(mut self, expr: Expression) -> Self {
        self.right = Some(expr);
        self
    }

    pub fn operand(mut self, op: Token) -> Self {
        self.op = Some(op);
        self
    }

    pub fn literal(mut self, literal: token::Literal) -> Self {
        self.literal = Some(literal);
        self
    }

    pub fn group(mut self, expr: Expression) -> Self {
        self.group = Some(expr);
        self
    }

    pub fn build(self) -> Result<Expression, Box<dyn Error>> {
        if let Some(literal) = self.literal {
            Ok(Expression::Literal(literal))
        } else if let Some(expr) = self.group {
            Ok(Expression::Group(Box::new(expr)))
        } else if let Some(right) = self.right {
            if let Some(op) = self.op {
                if let Some(left) = self.left {
                    return Ok(Expression::Binary(Box::new(left), op, Box::new(right)));
                } else {
                    return Ok(Expression::Unary(op, Box::new(right)));
                }
            }

            Err("operand is required!".into())
        } else {
            Err("invalid builder state".into())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{token::Literal, token_type::TokenType, LocationInfo};

    use super::*;

    #[test]
    fn expression_displayed_correctly() {
        let left_expr = ExpressionBuilder::new()
            .operand(Token {
                token_type: TokenType::Minus,
                lexeme: "-".to_string(),
                literal: token::Literal::String("Hello world!".chars().collect::<Vec<char>>()),
                loc: LocationInfo {
                    column: 0,
                    line: 0,
                    len: 1,
                },
            })
            .right_expression(
                ExpressionBuilder::new()
                    .literal(token::Literal::Number(123_f64))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();
        let right_expr = ExpressionBuilder::new()
            .group(
                ExpressionBuilder::new()
                    .literal(token::Literal::Number(45.67_f64))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();
        let expr = ExpressionBuilder::new()
            .left_expression(left_expr)
            .right_expression(right_expr)
            .operand(Token {
                token_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: Literal::String(['*'].into()),
                loc: LocationInfo {
                    column: 0,
                    line: 0,
                    len: 1,
                },
            })
            .build()
            .unwrap();

        assert_eq!("(* (- 123) (group 45.67))".to_string(), expr.display_text());
    }
}
