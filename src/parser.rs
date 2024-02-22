//! Parser for Lox
//!
//! Name        Operators   Associates
//! Equality    == !=       Left
//! Comparison  > >= < <=   Left
//! Term        - +         Left
//! Factor      / *         Left
//! Unary       ! -         Right
//!
//! Productions (Low -> High Precedence):
//! expression     → equality ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary
//!                | primary ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil"
//!                | "(" expression ")" ;

use crate::{
    errors::ParserError,
    expression::{Expression, ExpressionBuilder},
    token::{Literal, Token},
    token_type::TokenType,
};

pub struct Parser {
    tokens: Vec<Token>,
    curr: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, curr: 0 }
    }

    pub fn run(&mut self) -> Result<(), ParserError> {
        todo!()
    }

    pub fn parse(&mut self) -> Result<Expression, ParserError> {
        self.expression()
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            match self.peek().token_type {
                TokenType::Class
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => self.advance(),
            };
        }
    }

    fn consume(&mut self, _type: TokenType, msg: &str) -> Result<(), ParserError> {
        // TODO: Implement error reporting
        if self.check(&_type) {
            self.advance();
            return Ok(());
        }

        Err(ParserError {
            cause: msg.to_string(),
        })
    }

    fn expression(&mut self) -> Result<Expression, ParserError> {
        self.equality()
    }

    fn primary(&mut self) -> Result<Expression, ParserError> {
        let mut expr = ExpressionBuilder::new();
        if self.matches_token(vec![TokenType::False]) {
            expr = expr.literal(Literal::Boolean(false));
        } else if self.matches_token(vec![TokenType::True]) {
            expr = expr.literal(Literal::Boolean(true));
        } else if self.matches_token(vec![TokenType::Number, TokenType::String]) {
            expr = expr.literal(self.previous().literal);
        } else if self.matches_token(vec![TokenType::LeftParen]) {
            let group = self.expression()?;
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            expr = expr.group(group);
        } else {
            return Err(ParserError {
                cause: "Expect expression".into(),
            });
        }

        expr.build().map_err(|e| ParserError {
            cause: e.to_string(),
        })
    }

    fn unary(&mut self) -> Result<Expression, ParserError> {
        if self.matches_token(vec![TokenType::Bang, TokenType::Minus]) {
            let op = self.previous();
            let right = self.unary()?;
            return ExpressionBuilder::new()
                .operand(op)
                .right_expression(right)
                .build()
                .map_err(|e| ParserError {
                    cause: e.to_string(),
                });
        }
        self.primary()
    }

    fn factor(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.unary()?;

        while self.matches_token(vec![TokenType::Slash, TokenType::Star]) {
            let op = self.previous();
            let right = self.unary()?;
            expr = ExpressionBuilder::new()
                .left_expression(expr)
                .operand(op)
                .right_expression(right)
                .build()
                .map_err(|e| ParserError {
                    cause: e.to_string(),
                })?;
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.factor()?;

        while self.matches_token(vec![TokenType::Minus, TokenType::Plus]) {
            let op = self.previous();
            let right = self.factor()?;
            expr = ExpressionBuilder::new()
                .left_expression(expr)
                .operand(op)
                .right_expression(right)
                .build()
                .map_err(|e| ParserError {
                    cause: e.to_string(),
                })?;
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.term()?;

        while self.matches_token(vec![
            TokenType::GreaterEqual,
            TokenType::Greater,
            TokenType::LessEqual,
            TokenType::Less,
        ]) {
            let op = self.previous();
            let right = self.term()?;
            expr = ExpressionBuilder::new()
                .left_expression(expr)
                .operand(op)
                .right_expression(right)
                .build()
                .map_err(|e| ParserError {
                    cause: e.to_string(),
                })?;
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.comparison()?;

        while self.matches_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let op: Token = self.previous();
            let right = self.comparison()?;
            expr = ExpressionBuilder::new()
                .left_expression(expr)
                .operand(op)
                .right_expression(right)
                .build()
                .map_err(|e| ParserError {
                    cause: e.to_string(),
                })?;
        }

        Ok(expr)
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.curr += 1;
        }

        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.curr - 1].clone()
    }

    fn matches_token(&mut self, tokens: Vec<TokenType>) -> bool {
        for token_type in tokens.iter() {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, _type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == *_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.curr]
    }
}

#[cfg(test)]
mod test {
    use crate::scanner::Scanner;

    use super::*;

    #[test]
    fn syntax_tree_parsed_correctly() {
        let source = "(5 * 2) + 1 - 2".to_string();
        let mut scanner = Scanner::new(source);
        scanner.run().unwrap();
        eprintln!("{:?}", scanner.tokens);
        let mut parser = Parser::new(scanner.tokens);

        let expr = parser.parse().unwrap();

        assert_eq!("(- (+ (group (* 5 2)) 1) 2)", expr.display_text())
    }
}
