use std::fmt;

use crate::token::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct EvaluationError {
    msg: String,
    line: usize,
    column: usize,
}

impl EvaluationError {
    fn new(msg: &str, line: usize, column: usize) -> Self {
        Self {
            msg: msg.into(),
            line,
            column,
        }
    }
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "evaluation error: {} at line {} column {}",
            self.msg, self.line, self.column
        )
    }
}

#[derive(Clone, Debug)]
pub enum Expression {
    Unary(Token, Box<Expression>),
    Binary(Box<Expression>, Token, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(Token),
}

impl Expression {
    pub fn evaluate(&self) -> Result<String, EvaluationError> {
        match self {
            Expression::Grouping(expr) => expr.evaluate(),
            Expression::Unary(token, expr) => {
                let right = expr.evaluate()?;
                match token._type {
                    TokenType::Minus => {
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        Ok(format!("-{}", right))
                    }
                    TokenType::Not => {
                        let right = right.parse::<bool>().map_err(|_| {
                            EvaluationError::new("expected a boolean", token.line, token.column)
                        })?;
                        Ok(format!("{}", !right))
                    }
                    _ => Err(EvaluationError::new(
                        "unknown expression",
                        token.line,
                        token.column,
                    )),
                }
            }
            Expression::Binary(expr, token, rexpr) => {
                let left = expr.evaluate()?;
                let right = rexpr.evaluate()?;

                match token._type {
                    TokenType::Plus => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left + right))
                    }
                    TokenType::Minus => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left - right))
                    }
                    TokenType::Star => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left * right))
                    }
                    TokenType::Slash => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left / right))
                    }
                    TokenType::Or => {
                        let left = left.parse::<bool>().map_err(|_| {
                            EvaluationError::new("expected a boolean", token.line, token.column)
                        })?;
                        let right = right.parse::<bool>().map_err(|_| {
                            EvaluationError::new("expected a boolean", token.line, token.column)
                        })?;

                        Ok(format!("{}", left || right))
                    }
                    TokenType::Less => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left < right))
                    }
                    TokenType::LessEqual => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left <= right))
                    }
                    TokenType::Greater => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left > right))
                    }
                    TokenType::GreaterEqual => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left >= right))
                    }
                    TokenType::EqualEqual => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left == right))
                    }
                    TokenType::NotEqual => {
                        let left = left.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                        let right = right.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;

                        Ok(format!("{}", left != right))
                    }
                    TokenType::And => {
                        let left = left.parse::<bool>().map_err(|_| {
                            EvaluationError::new("expected a boolean", token.line, token.column)
                        })?;
                        let right = right.parse::<bool>().map_err(|_| {
                            EvaluationError::new("expected a boolean", token.line, token.column)
                        })?;

                        Ok(format!("{}", left && right))
                    }
                    _ => Err(EvaluationError::new(
                        "unknown expression",
                        token.line,
                        token.column,
                    )),
                }
            }
            Expression::Literal(token) => {
                match token._type {
                    TokenType::Number => {
                        token.lexeme.parse::<f32>().map_err(|_| {
                            EvaluationError::new("expected a number", token.line, token.column)
                        })?;
                    }
                    TokenType::True | TokenType::False => {
                        token.lexeme.parse::<bool>().map_err(|_| {
                            EvaluationError::new("expected a boolean", token.line, token.column)
                        })?;
                    }
                    _ => {}
                };

                Ok(token.lexeme.clone())
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

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, scanner::Scanner};

    fn evaluate_expression(expr: &str) -> String {
        let scanner = Scanner::new(expr.into()).unwrap();
        let mut parser = Parser::new(scanner.tokens);
        parser.parse_expression().unwrap().evaluate().unwrap()
    }

    #[test]
    fn calculation_expressions_are_evaluated_successfully() {
        let expression = "2 + 2 * 5";
        assert_eq!(evaluate_expression(expression), "12");

        let expression = "4 - 2";
        assert_eq!(evaluate_expression(expression), "2");

        let expression = "4 / 2 - 3";
        assert_eq!(evaluate_expression(expression), "-1");
    }

    #[test]
    fn unary_expressions_are_evaluated_successfully() {
        let expression = "!true";
        assert_eq!(evaluate_expression(expression), "false");

        let expression = "!false";
        assert_eq!(evaluate_expression(expression), "true");
    }

    #[test]
    fn conditional_expressions_are_evaluated_successfully() {
        let expression = "((2 * 6) < 12) || 4 > 5)";
        assert_eq!(evaluate_expression(expression), "false");

        let expression = "4 < 5 && 10 > 1";
        assert_eq!(evaluate_expression(expression), "true");
    }

    #[test]
    fn comparison_expressions_are_evaluated_succesfully() {
        let expression = "(2 + 4) <= 6";
        assert_eq!(evaluate_expression(expression), "true");

        let expression = "(2 + 4) < 6";
        assert_eq!(evaluate_expression(expression), "false");

        let expression = "(2 * 4) > 6";
        assert_eq!(evaluate_expression(expression), "true");

        let expression = "(4 / 2) >= 6";
        assert_eq!(evaluate_expression(expression), "false");

        let expression = "(2 + 4) == 6";
        assert_eq!(evaluate_expression(expression), "true");

        let expression = "(2 + 4) != 10";
        assert_eq!(evaluate_expression(expression), "true");
    }
}
