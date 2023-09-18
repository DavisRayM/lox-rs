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
///   expression -> equality
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
///   primary -> NUMBER | STRING | "true" | "false" | "(" expression ")" ;
use crate::{
    errors::{ExceptionType, ParserError},
    expression::Expression,
    token::{Token, TokenType},
};

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

    pub fn parse_expression(&mut self) -> ParserResult<Expression> {
        self.parse_equality()
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

    fn advance_if_match(&mut self, options: Vec<TokenType>) -> bool {
        let result = options.iter().any(|option| option == &self.peek()._type);
        if result {
            self.current += 1;
        }
        result
    }

    fn parse_equality(&mut self) -> ParserResult<Expression> {
        let mut expr = self.parse_comparison()?;

        while self.advance_if_match(vec![TokenType::NotEqual, TokenType::EqualEqual]) {
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
            TokenType::Or,
            TokenType::And,
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
        if self.advance_if_match(vec![TokenType::False]) {
            Ok(Expression::Literal("false".into()))
        } else if self.advance_if_match(vec![TokenType::True]) {
            Ok(Expression::Literal("true".into()))
        } else if self.advance_if_match(vec![TokenType::Number, TokenType::String]) {
            Ok(Expression::Literal(self.previous().lexeme))
        } else if self.advance_if_match(vec![TokenType::LeftParen]) {
            let expr = self.parse_expression()?;
            self.check_and_consume(TokenType::RightParen)?;
            Ok(Expression::Grouping(Box::new(expr)))
        } else {
            Err(ParserError::new(
                "invalid expression",
                &self.peek(),
                ExceptionType::RuntimeException,
            ))
        }
    }

    fn check_and_consume(&mut self, token_type: TokenType) -> ParserResult<()> {
        let token = &self.source[self.current];
        if token._type != token_type {
            return Err(ParserError::new(
                "expected {}",
                token,
                ExceptionType::RuntimeException,
            ));
        }

        self.current += 1;
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
            ("false", Expression::Literal("false".into()).into()),
            ("true", Expression::Literal("true".into()).into()),
            ("2000", Expression::Literal("2000".into()).into()),
            (
                "\"Hi there\"",
                Expression::Literal("Hi there".into()).into(),
            ),
            (
                "( 2000 )",
                Expression::Grouping(Box::new(Expression::Literal("2000".into()))).into(),
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
                    Box::new(Expression::Literal("1".into())),
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
                    Box::new(Expression::Literal("true".into())),
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
                    Box::new(Expression::Literal("2".into())),
                    Token {
                        line: 1,
                        lexeme: "*".into(),
                        _type: TokenType::Star,
                        column: 3,
                    },
                    Box::new(Expression::Literal("5".into())),
                )
                .into(),
            ),
            (
                "25 / 5",
                Expression::Binary(
                    Box::new(Expression::Literal("25".into())),
                    Token {
                        line: 1,
                        lexeme: "/".into(),
                        _type: TokenType::Slash,
                        column: 4,
                    },
                    Box::new(Expression::Literal("5".into())),
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
                    Box::new(Expression::Literal("4".into())),
                    Token {
                        line: 1,
                        lexeme: "==".into(),
                        _type: TokenType::EqualEqual,
                        column: 6,
                    },
                    Box::new(Expression::Literal("4".into())),
                )
                .into(),
            ),
            (
                "24.5 != 30",
                Expression::Binary(
                    Box::new(Expression::Literal("24.5".into())),
                    Token {
                        line: 1,
                        lexeme: "!=".into(),
                        _type: TokenType::NotEqual,
                        column: 6,
                    },
                    Box::new(Expression::Literal("30".into())),
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
                    Box::new(Expression::Literal("24.5".into())),
                    Token {
                        line: 1,
                        lexeme: "+".into(),
                        _type: TokenType::Plus,
                        column: 6,
                    },
                    Box::new(Expression::Literal("30".into())),
                )
                .into(),
            ),
            (
                "24.5 - 30",
                Expression::Binary(
                    Box::new(Expression::Literal("24.5".into())),
                    Token {
                        line: 1,
                        lexeme: "-".into(),
                        _type: TokenType::Minus,
                        column: 6,
                    },
                    Box::new(Expression::Literal("30".into())),
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
                "true || false",
                Expression::Binary(
                    Box::new(Expression::Literal("true".into())),
                    Token {
                        line: 1,
                        lexeme: "||".into(),
                        _type: TokenType::Or,
                        column: 6,
                    },
                    Box::new(Expression::Literal("false".into())),
                )
                .into(),
            ),
            (
                "true && true",
                Expression::Binary(
                    Box::new(Expression::Literal("true".into())),
                    Token {
                        line: 1,
                        lexeme: "&&".into(),
                        _type: TokenType::And,
                        column: 6,
                    },
                    Box::new(Expression::Literal("true".into())),
                )
                .into(),
            ),
            (
                "1 < 2",
                Expression::Binary(
                    Box::new(Expression::Literal("1".into())),
                    Token {
                        line: 1,
                        lexeme: "<".into(),
                        _type: TokenType::Less,
                        column: 6,
                    },
                    Box::new(Expression::Literal("2".into())),
                )
                .into(),
            ),
            (
                "2 <= 2",
                Expression::Binary(
                    Box::new(Expression::Literal("2".into())),
                    Token {
                        line: 1,
                        lexeme: "<=".into(),
                        _type: TokenType::LessEqual,
                        column: 6,
                    },
                    Box::new(Expression::Literal("2".into())),
                )
                .into(),
            ),
            (
                "3 > 4",
                Expression::Binary(
                    Box::new(Expression::Literal("3".into())),
                    Token {
                        line: 1,
                        lexeme: ">".into(),
                        _type: TokenType::Greater,
                        column: 6,
                    },
                    Box::new(Expression::Literal("4".into())),
                )
                .into(),
            ),
            (
                "4 >= 10",
                Expression::Binary(
                    Box::new(Expression::Literal("4".into())),
                    Token {
                        line: 1,
                        lexeme: ">=".into(),
                        _type: TokenType::GreaterEqual,
                        column: 6,
                    },
                    Box::new(Expression::Literal("10".into())),
                )
                .into(),
            ),
        ];

        assert_expression_scenarios(scenarios);
    }
}
