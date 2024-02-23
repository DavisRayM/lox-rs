use std::{
    fmt::Display,
    io,
    sync::{Arc, Mutex},
};

use crate::{
    environment::Environment,
    errors::{ParserError, RuntimeError},
    expression::Expression,
    statement::Statement,
    token::Literal,
    token_type::TokenType,
};

pub struct Interpreter<T: io::Write> {
    out: T,
    debug: bool,
    env: Arc<Mutex<Environment>>,
}

impl<T: io::Write> Interpreter<T> {
    pub fn new(out: T) -> Self {
        Self {
            out,
            debug: false,
            env: Arc::new(Mutex::new(Environment::new())),
        }
    }

    pub fn debug(&mut self, mode: bool) {
        self.debug = mode;
    }

    pub fn interpret(&mut self, stmts: Vec<Statement>) -> Result<(), RuntimeError> {
        stmts
            .iter()
            .try_for_each(|stmt| self.evaluate_statement(stmt))
    }

    fn evaluate_statement(&mut self, stmt: &Statement) -> Result<(), RuntimeError> {
        match stmt {
            Statement::Var(name, expr) => {
                let name = name.lexeme.clone();

                if let Some(expr) = expr {
                    let val = self.evaluate_expression(expr)?;
                    self.env.lock().unwrap().define(name, val)?;
                } else {
                    self.env.lock().unwrap().define(name, Literal::None)?;
                }
            }
            Statement::Expr(expr) => {
                self.evaluate_expression(expr)?;
            }
            Statement::Block(stmts) => {
                let previous = Arc::clone(&self.env);

                let env = Mutex::new(Environment::new());
                env.lock().unwrap().enclosing(Arc::clone(&self.env));
                self.env = Arc::new(env);

                stmts.iter().try_for_each(|s| self.evaluate_statement(s))?;
                self.env = previous;
            }
        }

        Ok(())
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Literal, RuntimeError> {
        match expr {
            Expression::Variable(name) => self.env.lock().unwrap().get(&name.lexeme),
            Expression::Assignment(name, expr) => {
                let val = self.evaluate_expression(expr)?;
                self.env
                    .lock()
                    .unwrap()
                    .assign(name.lexeme.clone(), val.clone())?;
                Ok(val)
            }
            Expression::Literal(literal) => Ok(literal.to_owned()),
            Expression::Group(expr) => self.evaluate_expression(expr),
            Expression::Unary(op, right) => {
                let right = self.evaluate_expression(right)?;
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
            Expression::Binary(left, op, right) => {
                let left = self.evaluate_expression(left)?;
                let right = self.evaluate_expression(right)?;

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

    fn print_to_output(&mut self, val: impl Display) -> Result<(), RuntimeError> {
        writeln!(&mut self.out, "{}", val).map_err(|e| RuntimeError {
            cause: format!("failed to print to console: {:?}", e),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::Parser, scanner::Scanner};

    use super::*;

    #[test]
    fn environment_tracks_variables() {
        let source = "var a = \"global a\";\nvar b = \"global b\";\nvar c = \"global c\";";

        let mut scanner = Scanner::new(source.trim().into());
        scanner.run().unwrap();
        let tokens = scanner.tokens;
        eprintln!("{:#?}", tokens);
        let mut parser = Parser::new(tokens, io::stderr());

        let mut intp = Interpreter::new(io::stderr());
        let stmts = parser.parse();
        eprintln!("{:#?}", stmts);
        intp.interpret(stmts).unwrap();

        assert_eq!(
            intp.env.lock().unwrap().get(&String::from("a")).unwrap(),
            Literal::String("global a".chars().collect::<Vec<char>>())
        );
    }

    #[test]
    fn nested_blocks_preserve_env() {
        let source = "var a = \"hello\";\n{\n    var a = \"world\";\n}\n";

        let mut scanner = Scanner::new(source.trim().into());
        scanner.run().unwrap();
        let tokens = scanner.tokens;
        eprintln!("{:#?}", tokens);
        let mut parser = Parser::new(tokens, io::stderr());

        let mut intp = Interpreter::new(io::stderr());
        let stmts = parser.parse();
        eprintln!("{:#?}", stmts);
        intp.interpret(stmts).unwrap();

        assert_eq!(
            intp.env.lock().unwrap().get(&String::from("a")).unwrap(),
            Literal::String("hello".chars().collect::<Vec<char>>())
        );
    }
}
