/// AST Parser for the Lox programming language
///
/// The AST Parser implementation uses recursive-descent parsing to parse lox
/// script files.
///
/// Meanings:
///   *Terminals* are tokens with literal values i.e TokenType::If,
///   TokenType::Number
///
///   *Non terminals* are tokens that run a certain scenario that returns
///   a literal value i.e TokenType::Plus, TokenType::NotEqual
///
/// Productions:
///   program -> declaration* EOF ;
///
///   declaration -> varDcl | statement;
///
///   declaration -> "var" IDENTIFIER ( "=" expression )? ";" ;
///
///   statement -> exprStmt ;
///
///   exprStmt -> expression ";" ;
///
///   expression -> assignment ;
///
///   assignment -> IDENTIFIER "=" assignment | equality;
///   
///   equality -> comparison ( ( "!=", "==" ) comparison) ;
///
///   comparison -> term ( ( ">", "<", ">=", "<=" ) factor )* ;
///
///   term -> factor ( ( "-", "+" ) factor)* ;
///
///   factor -> unary ( ( "/", "*" ) unary)* ;
///
///   unary -> ( "!", "-" ) unary | primary;
///
///   primary -> NUMBER | STRING | "true" | "false" | "(" expression ")"
///              | IDENTIFIER ;
use crate::{
    errors::{ExceptionType, ParserError},
    expression::Expression,
    literal::Literal,
    token::{Token, TokenType},
};

pub enum Statement {
    Expression(Expression),
    Variable(Expression),
    Assign(Token, Literal),
}

pub type ParserResult<T> = Result<T, ParserError>;

/// AST Parser for the Lox language
pub struct Parser {
    current: usize,
    source: Vec<Token>,
}

impl Parser {
    pub fn new(source: Vec<Token>) -> Self {
        Self { source, current: 0 }
    }

    pub fn parse(&mut self) -> ParserResult<Vec<Statement>> {
        let mut statements: Vec<Statement> = Vec::new();
        while self.current < self.source.len() {
            statements.push(self.parse_declaration()?);
        }

        Ok(statements)
    }

    fn parse_declaration(&mut self) -> ParserResult<Statement> {
        if self.matches(vec![TokenType::Let]) {
            self.parse_variable()
        } else {
            self.parse_statement()
        }
    }

    fn parse_variable(&mut self) -> ParserResult<Statement> {
        if !self.matches(vec![TokenType::Identifier]) {
            Err(ParserError::new(
                "expected an identifier",
                &self.peek(),
                ExceptionType::RuntimeException,
            ))
        } else {
            self.consume();
            self.check_and_consume(TokenType::Equal)?;
            let initializer = self.parse_expression()?;
            self.check_and_consume(TokenType::SemiColon)?;
            Ok(Statement::Variable(initializer))
        }
    }

    pub fn parse_statement(&mut self) -> ParserResult<Statement> {
        let expr = self.parse_expression()?;
        self.check_and_consume(TokenType::SemiColon)?;
        Ok(Statement::Expression(expr))
    }

    pub fn parse_assignment(&mut self) -> ParserResult<Expression> {
        let expr = self.parse_equality()?;

        if self.matches(vec![TokenType::Equal]) {
            let name = self.previous();
            let equals = self.consume();
            let rexpr = self.parse_assignment()?;

            match rexpr {
                Expression::Variable(_) => Ok(Expression::Assignment(
                    name.clone(),
                    rexpr.evaluate().map_err(|_| {
                        ParserError::new(
                            "invalid assignment",
                            &name,
                            ExceptionType::RuntimeException,
                        )
                    })?,
                )),
                _ => Err(ParserError::new(
                    "invalid assignment target",
                    &equals,
                    ExceptionType::RuntimeException,
                )),
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_expression(&mut self) -> ParserResult<Expression> {
        self.parse_assignment()
    }

    fn peek(&self) -> Token {
        if self.current < self.source.len() {
            self.source[self.current].clone()
        } else {
            self.previous()
        }
    }

    fn previous(&self) -> Token {
        self.source[self.current - 1].clone()
    }

    fn matches(&self, options: Vec<TokenType>) -> bool {
        options.iter().any(|option| option == &self.peek()._type)
    }

    fn consume(&mut self) -> Token {
        let token = self.peek();
        self.current += 1;
        token
    }

    fn advance_if_match(&mut self, options: Vec<TokenType>) -> bool {
        let result = self.matches(options);
        if result {
            self.consume();
        }
        result
    }

    fn parse_equality(&mut self) -> ParserResult<Expression> {
        let mut expr = self.parse_comparison()?;

        while self.advance_if_match(vec![
            TokenType::NotEqual,
            TokenType::EqualEqual,
            TokenType::Or,
            TokenType::And,
        ]) {
            let operator = self.previous();
            let r_expr = self.parse_comparison()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(r_expr));
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> ParserResult<Expression> {
        let mut expr = self.parse_term()?;

        while self.advance_if_match(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::EqualEqual,
        ]) {
            let operator = self.previous();
            let rexpr = self.parse_term()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(rexpr));
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> ParserResult<Expression> {
        let mut expr = self.parse_factor()?;

        while self.advance_if_match(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let rexpr = self.parse_factor()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(rexpr));
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> ParserResult<Expression> {
        let mut expr = self.parse_unary()?;

        while self.advance_if_match(vec![TokenType::Slash, TokenType::Star, TokenType::Equal]) {
            let operator = self.previous();
            let rexpr = self.parse_unary()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(rexpr));
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> ParserResult<Expression> {
        if self.advance_if_match(vec![TokenType::Not, TokenType::Minus]) {
            let operator = self.previous();
            let rexpr = self.parse_unary()?;
            Ok(Expression::Unary(operator, Box::new(rexpr)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> ParserResult<Expression> {
        if self.matches(vec![
            TokenType::False,
            TokenType::True,
            TokenType::Number,
            TokenType::String,
        ]) {
            Ok(Expression::Literal(self.consume()))
        } else if self.advance_if_match(vec![TokenType::LeftParen]) {
            let expr = self.parse_expression()?;
            self.check_and_consume(TokenType::RightParen)?;
            Ok(Expression::Grouping(Box::new(expr)))
        } else {
            Ok(Expression::Variable(self.consume()))
        }
    }

    fn check_and_consume(&mut self, token_type: TokenType) -> ParserResult<()> {
        let token = self.peek();
        if token._type != token_type {
            return Err(ParserError::new(
                &format!("expected {:?}", token_type),
                &token,
                ExceptionType::RuntimeException,
            ));
        }

        self.consume();
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::scanner::Scanner;

    fn assert_expression_scenarios(scenarios: Vec<(&str, String)>) {
        for (scenario, expected) in scenarios {
            let tokens = Scanner::new(scenario.into()).unwrap().tokens;
            let mut parser = Parser::new(tokens);
            let expression: String = parser.parse_expression().unwrap().into();

            assert_eq!(expression, expected);
        }
    }

    #[test]
    fn parses_primary_expressions() {
        let scenarios: Vec<(&str, String)> = vec![
            (
                "false",
                Expression::Literal(Token::new("false", 1, 1, TokenType::False)).into(),
            ),
            (
                "true",
                Expression::Literal(Token::new("true", 1, 1, TokenType::True)).into(),
            ),
            (
                "2000",
                Expression::Literal(Token::new("2000", 1, 1, TokenType::Number)).into(),
            ),
            (
                "\"Hi there\"",
                Expression::Literal(Token::new("Hi there", 1, 1, TokenType::String)).into(),
            ),
            (
                "( 2000 )",
                Expression::Grouping(Box::new(Expression::Literal(Token::new(
                    "2000",
                    1,
                    1,
                    TokenType::Number,
                ))))
                .into(),
            ),
        ];
        assert_expression_scenarios(scenarios);
    }

    #[test]
    fn parses_unary_expressions() {
        let scenarios: Vec<(&str, String)> = vec![
            (
                "-1",
                Expression::Unary(
                    Token {
                        line: 1,
                        lexeme: "-".into(),
                        _type: TokenType::Minus,
                        column: 1,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "1",
                        1,
                        2,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
            (
                "!true",
                Expression::Unary(
                    Token {
                        line: 1,
                        lexeme: "!".into(),
                        _type: TokenType::Not,
                        column: 1,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "true",
                        1,
                        2,
                        TokenType::True,
                    ))),
                )
                .into(),
            ),
        ];

        assert_expression_scenarios(scenarios);
    }

    #[test]
    fn parses_factor_expressions() {
        let scenarios: Vec<(&str, String)> = vec![
            (
                "2 * 5",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "2",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "*".into(),
                        _type: TokenType::Star,
                        column: 3,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "5",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
            (
                "25 / 5",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "25",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "/".into(),
                        _type: TokenType::Slash,
                        column: 4,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "5",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
        ];

        assert_expression_scenarios(scenarios);
    }

    #[test]
    fn parses_equality_expressions() {
        let scenarios: Vec<(&str, String)> = vec![
            (
                "4 == 4",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "4",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "==".into(),
                        _type: TokenType::EqualEqual,
                        column: 3,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "4",
                        1,
                        5,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
            (
                "24.5 != 30",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "24.5",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "!=".into(),
                        _type: TokenType::NotEqual,
                        column: 6,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "30",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
        ];

        assert_expression_scenarios(scenarios);
    }

    #[test]
    fn parses_terminal_expressions() {
        let scenarios: Vec<(&str, String)> = vec![
            (
                "24.5 + 30",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "24.5",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "+".into(),
                        _type: TokenType::Plus,
                        column: 6,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "30",
                        1,
                        8,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
            (
                "24.5 - 30",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "24.5",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "-".into(),
                        _type: TokenType::Minus,
                        column: 6,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "30",
                        1,
                        8,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
        ];

        assert_expression_scenarios(scenarios);
    }

    #[test]
    fn parses_comparison_expressions() {
        let scenarios: Vec<(&str, String)> = vec![
            (
                "true || 3 < 2",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "true",
                        1,
                        1,
                        TokenType::True,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "||".into(),
                        _type: TokenType::Or,
                        column: 6,
                    },
                    Box::new(Expression::Binary(
                        Box::new(Expression::Literal(Token::new(
                            "3",
                            0,
                            0,
                            TokenType::Number,
                        ))),
                        Token::new("<", 0, 0, TokenType::Less),
                        Box::new(Expression::Literal(Token::new(
                            "2",
                            0,
                            0,
                            TokenType::Number,
                        ))),
                    )),
                )
                .into(),
            ),
            (
                "true && true",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "true",
                        1,
                        1,
                        TokenType::True,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "&&".into(),
                        _type: TokenType::And,
                        column: 6,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "true",
                        1,
                        9,
                        TokenType::True,
                    ))),
                )
                .into(),
            ),
            (
                "1 < 2",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "1",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "<".into(),
                        _type: TokenType::Less,
                        column: 3,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "2",
                        1,
                        5,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
            (
                "2 <= 2",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "2",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: "<=".into(),
                        _type: TokenType::LessEqual,
                        column: 3,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "2",
                        1,
                        5,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
            (
                "3 > 4",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "3",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: ">".into(),
                        _type: TokenType::Greater,
                        column: 3,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "4",
                        1,
                        5,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
            (
                "4 >= 10",
                Expression::Binary(
                    Box::new(Expression::Literal(Token::new(
                        "4",
                        1,
                        1,
                        TokenType::Number,
                    ))),
                    Token {
                        line: 1,
                        lexeme: ">=".into(),
                        _type: TokenType::GreaterEqual,
                        column: 3,
                    },
                    Box::new(Expression::Literal(Token::new(
                        "10",
                        1,
                        5,
                        TokenType::Number,
                    ))),
                )
                .into(),
            ),
        ];

        assert_expression_scenarios(scenarios);
    }
}
