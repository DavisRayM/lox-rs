use std::error::Error;

use crate::token::{self, Literal, Token};

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Logical(Box<Expression>, Token, Box<Expression>),
    Group(Box<Expression>),
    Literal(token::Literal),
    Unary(Token, Box<Expression>),
    Variable(Token),
    Assignment(Token, Box<Expression>),
}

impl Expression {
    #[allow(dead_code)]
    pub(crate) fn display_text(&self) -> String {
        match self {
            Self::Variable(var) => {
                format!("(var {})", var.lexeme)
            }
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
            Self::Assignment(name, expr) => {
                format!("({} = {})", name.lexeme, expr.display_text())
            }
            Self::Binary(left, op, right) | Self::Logical(left, op, right) => {
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
    variable: Option<Token>,
}

impl ExpressionBuilder {
    pub fn new() -> Self {
        ExpressionBuilder {
            left: None,
            right: None,
            op: None,
            literal: None,
            group: None,
            variable: None,
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

    pub fn variable(mut self, name: Token) -> Self {
        self.variable = Some(name);
        self
    }

    pub fn build(self) -> Result<Expression, Box<dyn Error>> {
        if let Some(literal) = self.literal {
            Ok(Expression::Literal(literal))
        } else if let Some(var) = self.variable {
            if let Some(expr) = self.right {
                Ok(Expression::Assignment(var, Box::new(expr)))
            } else {
                Ok(Expression::Variable(var))
            }
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
