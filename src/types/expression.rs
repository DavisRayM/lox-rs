use crate::{Environment, EvaluationError, Literal, Token, TokenType};

#[derive(Clone, Debug)]
pub enum Expression {
    Unary(Token, Box<Expression>),
    Binary(Box<Expression>, Token, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(Token),
    Variable(Token),
    Assignment(Token, Box<Expression>),
}

impl Expression {
    pub fn evaluate(&self, environment: &Environment) -> Result<Literal, EvaluationError> {
        match self {
            Expression::Grouping(expr) => expr.evaluate(environment),
            Expression::Variable(token) => {
                if token._type == TokenType::Identifier {
                    if let Some(literal) = environment.get(token.lexeme.clone()) {
                        Ok(literal)
                    } else {
                        Ok(Literal::Variable(token.lexeme.clone()))
                    }
                } else {
                    Err(EvaluationError::new(
                        "unexpected variable type",
                        token.line,
                        token.column,
                    ))
                }
            }
            Expression::Assignment(token, expr) => {
                if let Expression::Variable(_) = expr.as_ref() {
                    expr.evaluate(environment)
                } else {
                    Err(EvaluationError::new(
                        "unqualified variable name",
                        token.line,
                        token.column,
                    ))
                }
            }
            Expression::Unary(token, expr) => {
                let right = expr.evaluate(environment)?;
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
                let left = expr.evaluate(environment)?;
                let right = rexpr.evaluate(environment)?;
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
    use crate::{get_statement_string, parser::Parser, scanner::Scanner};

    fn evaluate_statement(expr: &str) -> String {
        let scanner = Scanner::new(expr.into()).unwrap();
        let mut parser = Parser::new(scanner.tokens, true);
        let statements = parser.parse().unwrap();
        let mut out = String::new();

        for statement in statements {
            out.push_str(&get_statement_string(statement));
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
