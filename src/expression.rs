use std::fmt;

use crate::literal::Literal;
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
    Variable(Token),
    Assignment(Token, Literal),
}

impl Expression {
    pub fn evaluate(&self) -> Result<Literal, EvaluationError> {
        match self {
            Expression::Grouping(expr) => expr.evaluate(),
            Expression::Variable(token) => {
                if token._type == TokenType::Identifier {
                    Ok(Literal::Variable(token.lexeme.clone()))
                } else {
                    Err(EvaluationError::new(
                        "unexpected variable type",
                        token.line,
                        token.column,
                    ))
                }
            }
            Expression::Assignment(token, literal) => {
                if token._type == TokenType::Identifier {
                    let literal = literal.clone();
                    Ok(Literal::Assignment(token.lexeme.clone(), Box::new(literal)))
                } else {
                    Err(EvaluationError::new(
                        "unqualified variable name",
                        token.line,
                        token.column,
                    ))
                }
            }
            Expression::Unary(token, expr) => {
                let right = expr.evaluate()?;
                match token._type {
                    TokenType::Minus => {
                        if let Literal::Number(value) = right {
                            Ok(Literal::Number(-value))
                        } else {
                            Err(EvaluationError::new(
                                "expected a number",
                                token.line,
                                token.column,
                            ))
                        }
                    }
                    TokenType::Not => {
                        if let Literal::Boolean(value) = right {
                            Ok(Literal::Boolean(!value))
                        } else {
                            Err(EvaluationError::new(
                                "expected a number",
                                token.line,
                                token.column,
                            ))
                        }
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
                let values = (left, right);

                match values {
                    (Literal::Number(left), Literal::Number(right)) => match token._type {
                        TokenType::Plus => Ok(Literal::Number(left + right)),
                        TokenType::Minus => Ok(Literal::Number(left - right)),
                        TokenType::Star => Ok(Literal::Number(left * right)),
                        TokenType::Slash => Ok(Literal::Number(left / right)),
                        TokenType::LessEqual => Ok(Literal::Boolean(left <= right)),
                        TokenType::Less => Ok(Literal::Boolean(left < right)),
                        TokenType::GreaterEqual => Ok(Literal::Boolean(left >= right)),
                        TokenType::Greater => Ok(Literal::Boolean(left > right)),
                        TokenType::NotEqual => Ok(Literal::Boolean(left != right)),
                        TokenType::EqualEqual => Ok(Literal::Boolean(left == right)),
                        _ => todo!(),
                    },
                    (Literal::Boolean(left), Literal::Boolean(right)) => match token._type {
                        TokenType::Or => Ok(Literal::Boolean(left || right)),
                        TokenType::And => Ok(Literal::Boolean(left && right)),
                        _ => todo!(),
                    },
                    _ => Err(EvaluationError::new(
                        "unknown operator",
                        token.line,
                        token.column,
                    )),
                }
            }
            Expression::Literal(token) => match token._type {
                TokenType::Number => {
                    let value = token.lexeme.parse::<f32>().map_err(|_| {
                        EvaluationError::new("expected a number", token.line, token.column)
                    })?;
                    Ok(Literal::Number(value))
                }
                TokenType::True | TokenType::False => {
                    let value = token.lexeme.parse::<bool>().map_err(|_| {
                        EvaluationError::new("expected a boolean", token.line, token.column)
                    })?;
                    Ok(Literal::Boolean(value))
                }
                TokenType::String => {
                    let value = token.lexeme.clone();
                    Ok(Literal::String(value))
                }
                _ => Err(EvaluationError::new(
                    "unknown value",
                    token.line,
                    token.column,
                )),
            },
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
            Expression::Literal(token) | Expression::Variable(token) => token.lexeme,
            Expression::Assignment(token, literal) => {
                format!("({} = {:?})", token.lexeme.clone(), literal)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::{Parser, Statement},
        scanner::Scanner,
    };

    fn evaluate_statement(expr: &str) -> String {
        let scanner = Scanner::new(expr.into()).unwrap();
        let mut parser = Parser::new(scanner.tokens);
        let statements = parser.parse().unwrap();
        let mut out = String::new();

        for statement in statements {
            match statement {
                Statement::Assign(token, literal) => {
                    let str_rep: String = literal.into();
                    out.push_str(&format!("let {} = {}", token.lexeme, str_rep));
                }
                Statement::Variable(expr) => {
                    let str_rep: String = expr.evaluate().unwrap().into();
                    out.push_str(&str_rep)
                }
                Statement::Expression(expr) => {
                    let str_rep: String = expr.evaluate().unwrap().into();
                    out.push_str(&str_rep)
                }
            }
        }

        out
    }

    #[test]
    fn calculation_expressions_are_evaluated_successfully() {
        let expression = "2 + 2 * 5;";
        assert_eq!(evaluate_statement(expression), "12");

        let expression = "4 - 2;";
        assert_eq!(evaluate_statement(expression), "2");

        let expression = "4 / 2 - 3;";
        assert_eq!(evaluate_statement(expression), "-1");
    }

    #[test]
    fn unary_expressions_are_evaluated_successfully() {
        let expression = "!true;";
        assert_eq!(evaluate_statement(expression), "false");

        let expression = "!false;";
        assert_eq!(evaluate_statement(expression), "true");
    }

    #[test]
    fn conditional_expressions_are_evaluated_successfully() {
        let expression = "(2 * 6) < 12 || 4 > 5;";
        assert_eq!(evaluate_statement(expression), "false");

        let expression = "4 < 5 && 10 > 1;";
        assert_eq!(evaluate_statement(expression), "true");
    }

    #[test]
    fn comparison_expressions_are_evaluated_succesfully() {
        let expression = "(2 + 4) <= 6;";
        assert_eq!(evaluate_statement(expression), "true");

        let expression = "(2 + 4) < 6;";
        assert_eq!(evaluate_statement(expression), "false");

        let expression = "(2 * 4) > 6;";
        assert_eq!(evaluate_statement(expression), "true");

        let expression = "(4 / 2) >= 6;";
        assert_eq!(evaluate_statement(expression), "false");

        let expression = "(2 + 4) == 6;";
        assert_eq!(evaluate_statement(expression), "true");

        let expression = "(2 + 4) != 10;";
        assert_eq!(evaluate_statement(expression), "true");
    }
}
