use crate::token::{Token, TokenType};

#[derive(Clone, Debug)]
pub enum Expression {
    Unary(Token, Box<Expression>),
    Binary(Box<Expression>, Token, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(Token),
}

impl Expression {
    pub fn evaluate(&self) -> String {
        match self {
            Expression::Literal(val) => val.lexeme.clone(),
            Expression::Grouping(expr) => expr.evaluate().to_owned(),
            Expression::Unary(operator, expr) => {
                let right = expr.evaluate();
                match operator._type {
                    TokenType::Minus => {
                        format!("-{}", right)
                    }
                    _ => "".into(),
                }
            }
            Expression::Binary(expr, operator, rexpr) => {
                let left = expr.evaluate().parse::<f32>().unwrap();
                let right = rexpr.evaluate().parse::<f32>().unwrap();

                match operator._type {
                    TokenType::Minus => {
                        format!("{}", left - right)
                    }
                    TokenType::Slash => {
                        format!("{}", left / right)
                    }
                    TokenType::Star => {
                        format!("{}", left * right)
                    }
                    TokenType::Plus => {
                        format!("{}", left + right)
                    }
                    _ => "".into(),
                }
            }
        }
    }
}

impl From<Expression> for String {
    fn from(val: Expression) -> String {
        match val {
            Expression::Unary(token, expr) => {
                let expr: String = expr.as_ref().to_owned().into();
                format!("({} {})", token.lexeme, expr)
            }
            Expression::Binary(expr, token, r_expr) => {
                let expr: String = expr.as_ref().to_owned().into();
                let r_expr: String = r_expr.as_ref().to_owned().into();
                format!("({} {} {})", expr, token.lexeme, r_expr)
            }
            Expression::Grouping(expr) => {
                let expr: String = expr.as_ref().to_owned().into();
                format!("(group {})", expr)
            }
            Expression::Literal(token) => token.lexeme,
        }
    }
}
