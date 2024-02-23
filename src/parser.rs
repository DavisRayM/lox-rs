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
//! program        → declaration* EOF ;
//! declaration    → varDecl | statement ;
//! varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
//! statement      → varStmt | exprStmt | block ;
//! exprStmt       → expression ";" ;
//! block          → "{" declaration "}" ;
//! expression     → equality ;
//! assignment     → IDENTIFIER "=" assignment | equality ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary
//!                | primary ;
//! primary        → NUMBER | STRING | "true" | "false" | "nil" | IDENTIFIER
//!                | "(" expression ")" ;

use std::io;

use crate::{
    errors::ParserError,
    expression::{Expression, ExpressionBuilder},
    statement::Statement,
    token::{Literal, Token},
    token_type::TokenType,
};

/// Language parser
///
/// Parses a list of tokens into an Expression tree that can then be evaluated.
pub struct Parser<T: io::Write> {
    tokens: Vec<Token>,
    curr: usize,
    out: T,
}

impl<T: io::Write> Parser<T> {
    /// Create a new parser that can be used to generate an expression tree
    pub fn new(tokens: Vec<Token>, out: T) -> Self {
        Self {
            tokens,
            curr: 0,
            out,
        }
    }

    /// Generates an expression tree from the configured tokens list
    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
        let mut error = false;

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    error = true;
                    writeln!(self.out, "{}", e).unwrap();
                    self.synchronize();
                }
            }
        }

        if !error {
            statements
        } else {
            statements.clear();
            statements
        }
    }

    fn declaration(&mut self) -> Result<Statement, ParserError> {
        if self.matches_token(vec![TokenType::Var]) {
            self.var_declaration()
        } else if self.matches_token(vec![TokenType::LeftBrace]) {
            self.block()
        } else {
            self.statement()
        }
    }

    fn block(&mut self) -> Result<Statement, ParserError> {
        let mut stmts = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "expect '}' after block")?;
        Ok(Statement::Block(stmts))
    }

    fn var_declaration(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenType::Identifier, "expect a variable name")?;

        let mut expr = None;
        if self.matches_token(vec![TokenType::Equal]) {
            expr = Some(self.expression()?);
        }

        self.consume(TokenType::Semicolon, "expect ';' after declaration")?;
        Ok(Statement::Var(name, expr))
    }

    fn statement(&mut self) -> Result<Statement, ParserError> {
        self.expr_statement()
    }

    fn expr_statement(&mut self) -> Result<Statement, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "expected ';' after expression.")?;
        Ok(Statement::Expr(expr))
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

    fn consume(&mut self, _type: TokenType, msg: &str) -> Result<Token, ParserError> {
        if self.check(&_type) {
            return Ok(self.advance());
        }

        Err(ParserError {
            cause: msg.to_string(),
        })
    }

    fn expression(&mut self) -> Result<Expression, ParserError> {
        self.equality()
    }

    fn assignment(&mut self) -> Result<Expression, ParserError> {
        let expr = self.equality()?;

        if self.matches_token(vec![TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expression::Variable(name) => Ok(Expression::Assignment(name, Box::new(value))),
                _ => Err(ParserError {
                    cause: format!(
                        "invalid assignment target at {} {}",
                        equals.loc.column, equals.loc.line
                    ),
                }),
            }
        } else {
            Ok(expr)
        }
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
        } else if self.matches_token(vec![TokenType::Identifier]) {
            expr = expr.variable(self.previous());
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
        let source = "(5 * 2) + 1 - 2;".to_string();
        let mut scanner = Scanner::new(source);
        scanner.run().unwrap();

        let sink = io::sink();
        let mut parser = Parser::new(scanner.tokens, sink);
        let stmt = parser.parse();

        assert_eq!(1, stmt.len());
    }
}
