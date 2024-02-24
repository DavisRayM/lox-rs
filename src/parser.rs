//! Parser module
//!
//! This module contains a parser implementation for the language.
//!
//! See [Parser] for more detailed information on how to parse symbols in the language.
//!
//! # Expression associations
//!
//! ```markdown
//! Name        Operators   Associates
//! Equality    == !=       Left
//! Comparison  > >= < <=   Left
//! Term        - +         Left
//! Factor      / *         Left
//! Unary       ! -         Right
//! Assignment  =           Right
//! ```
//!
//! *Note*: The direction of association specifies how that particular expression is
//! evaluated i.e the assignment expression is evaluated from right to
//! left; we first evaluate the `expression` on the right of the equal then we evaluate the `=`.
//!
//! # Productions (Low → High Precedence)
//!
//! ```markdown
//! program        → declaration* EOF ;
//! declaration    → varDecl
//!                  | statement ;
//! varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
//! statement      → exprStmt
//!                  | break
//!                  | printStmt
//!                  | ifStmt
//!                  | whileStmt
//!                  | forStmt
//!                  | block ;
//! forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
//!                  expression? ";"
//!                  expression? ")" statement ;
//! whileStmt      → "while" "(" expression ")" statement ;
//! ifStmt         → "if" "(" expressiong ")" statement ("else" statement)? ;
//! printStmt      → "print" expression ";" ;
//! exprStmt       → expression ";" ;
//! block          → "{" declaration "}" ;
//! expression     → assignment ;
//! assignment     → IDENTIFIER "=" assignment | logical_or ;
//! logic_or       → logic_and ( "or" logic_and )* ;
//! logic_and      → equality ( "and" equality )* ;
//! equality       → comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           → factor ( ( "-" | "+" ) factor )* ;
//! factor         → unary ( ( "/" | "*" ) unary )* ;
//! unary          → ( "!" | "-" ) unary
//!                  | call ;
//! call           → primary ( "(" arguments? ")" ) * ;
//! arguments      → expression ( "," expression)* ;
//! primary        → NUMBER
//!                  | STRING
//!                  | "true"
//!                  | "false"
//!                  | "nil"
//!                  | IDENTIFIER
//!                  | "(" expression ")" ;
//! ```

use std::io;

use crate::{
    errors::ParserError, expression::ExpressionBuilder, token::TokenBuilder, Expression, Literal,
    Statement, Token, TokenType,
};

/// Symbol parser
///
/// Parses a list of [symbols](Token) into valid [statements](Statement).
///
/// ```
/// use lox_rs::{Token, Parser, Statement};
///
/// let tokens: Vec<Token> = Vec::new();
/// let mut parser = Parser::new(tokens, std::io::stdout(), true);
/// let stmts: Vec<Statement> = parser.parse();
/// ```
pub struct Parser<T: io::Write> {
    tokens: Vec<Token>,
    curr: usize,
    out: T,
    in_loop: bool,
    /// Whether the parser should allow missing [Semicolon](TokenType) at the
    /// end of statements.
    pub strict: bool,
}

impl<T: io::Write> Parser<T> {
    /// Creates a new [`Parser<T>`] object. The new parser outputs debug information
    /// to the [Write](std::io::Write) object passed.
    pub fn new(tokens: Vec<Token>, out: T, strict: bool) -> Self {
        Self {
            tokens,
            curr: 0,
            out,
            strict,
            in_loop: false,
        }
    }

    /// Parses the symbols[Token] held by the parser into valid statements
    ///
    /// This will return an empty vector if the symbols do not produce valid
    /// statements. Any parsing errors encounter will be written to the
    /// configured [Write](std::io::Write) object.
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

    /// Produces a declaration. A declaration is either a
    /// variable declaration or a statement.
    fn declaration(&mut self) -> Result<Statement, ParserError> {
        if self.matches_token(vec![TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    /// Produces a [variable statement](Statement::Var).
    ///
    /// A variable statement is produced by the following symbols:
    /// "[Identifier](TokenType) ([Equal](TokenType) [Expression])?
    /// [Semicolon](TokenType)"
    ///
    /// # Errors
    ///
    /// If this function encounters a missing [Identifier](TokenType) or
    /// [Semicolon](TokenType)(While in strict mode. See: [Parser]) a [ParserError] will be
    /// returned.
    fn var_declaration(&mut self) -> Result<Statement, ParserError> {
        let name = self.consume(TokenType::Identifier, "expect a variable name")?;

        let mut expr = None;
        if self.matches_token(vec![TokenType::Equal]) {
            expr = Some(self.expression()?);
        }

        self.consume(TokenType::Semicolon, "expect ';' after declaration")?;
        Ok(Statement::Var(name, expr))
    }

    /// Produces a statement. The supported statements can be found
    /// in the [Statement] enum.
    ///
    /// This is where symbols that make up the statement production should be parsed. See the top
    /// level module documentation for more information.
    fn statement(&mut self) -> Result<Statement, ParserError> {
        if self.matches_token(vec![TokenType::Print]) {
            return self.print_statement();
        } else if self.matches_token(vec![TokenType::If]) {
            return self.if_statement();
        } else if self.matches_token(vec![TokenType::For]) {
            return self.for_statement();
        } else if self.matches_token(vec![TokenType::While]) {
            return self.while_statement();
        } else if self.matches_token(vec![TokenType::LeftBrace]) {
            return self.block();
        } else if self.matches_token(vec![TokenType::Break]) {
            if !self.in_loop {
                return Err(ParserError {
                    cause: "break can not be used outside a loop".into(),
                });
            }
            self.consume(TokenType::Semicolon, "expect ';' after break")?;
            return Ok(Statement::Break);
        }
        self.expr_statement()
    }

    /// Produces a block statement. This function is called after matching a
    /// [LeftBrace](TokenType) symbol during the `statement()` function.
    ///
    /// # Errors
    ///
    /// This functions errors out if a [RightBrace](TokenType) is not encountered before the parser
    /// reaches the end.
    fn block(&mut self) -> Result<Statement, ParserError> {
        let mut stmts = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "expect '}' after block")?;
        Ok(Statement::Block(stmts))
    }

    /// Produces a for statement. This function should be called after matching a [For](TokenType)
    /// symbol during the `statement()` function.
    ///
    /// This just desugars the for declaration into a [While](Statement::While) which is pretty
    /// cool. It's a good example of how syntatic sugar is added in programming languages without
    /// having to modify the interpreter in any way since it should probably already know how to
    /// handle while statements.
    ///
    /// # Supported syntax
    ///
    /// ```lox
    /// for (var i = 0; i < 5; i++) {
    ///     ...
    /// }
    ///
    /// var temp = 0;
    /// for (;temp < 2; temp++) {
    ///     ...
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - If a [LeftParen](TokenType) is not encountered immediately after calling this function.
    /// - If a [Semicolon](TokenType) is not encountered after the initializer & condition.
    fn for_statement(&mut self) -> Result<Statement, ParserError> {
        let mut loop_state_set = false;
        if !self.in_loop {
            self.in_loop = true;
            loop_state_set = true;
        }
        self.consume(TokenType::LeftParen, "expect '(' after 'for'")?;

        let initializer: Option<Statement>;
        if self.matches_token(vec![TokenType::Semicolon]) {
            initializer = None
        } else if self.matches_token(vec![TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expr_statement()?);
        }

        let mut condition: Expression = Expression::Literal(Literal::Boolean(true));
        if !self.check(&TokenType::Semicolon) {
            condition = self.expression()?;
        }
        self.consume(TokenType::Semicolon, "expect ';' after loop condition")?;

        let mut increment: Option<Expression> = None;
        if !self.check(&TokenType::RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "expect ')' after 'for' clauses")?;
        let mut body = self.statement()?;

        // Wrap the body of the for loop and increment function into a block
        // the interpreter will execute them in that order
        if let Some(incr) = increment {
            body = Statement::Block(vec![body, Statement::Expr(incr)]);
        }

        // Wrap the block created above in a while loop that's executed
        // as long as the condition is true
        body = Statement::While(condition, Box::new(body));

        // Lastly, wrap the initializer if any into a block that's only executed
        // once before the while loop starts. This also takes advantage of the
        // block context available
        if let Some(init) = initializer {
            body = Statement::Block(vec![init, body]);
        }

        if loop_state_set {
            self.in_loop = false;
        }
        Ok(body)
    }

    /// Parses a while statement.
    ///
    /// Supported syntax:
    ///
    /// ```lox
    /// while(cond) statement
    ///
    /// while(cond) {
    ///    ...
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - If a [LeftParen](TokenType) is not encountered
    /// - If a [RightParen](TokenType) is not encountered after processing the condition expression
    fn while_statement(&mut self) -> Result<Statement, ParserError> {
        let mut loop_state_set = false;
        if !self.in_loop {
            self.in_loop = true;
            loop_state_set = true;
        }
        self.consume(TokenType::LeftParen, "expect '(' after while condition")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "expect ')' after while condition")?;
        let statement = self.statement()?;

        if loop_state_set {
            self.in_loop = false;
        }
        Ok(Statement::While(cond, Box::new(statement)))
    }

    /// Parses an if statement.
    ///
    /// Supported syntax:
    ///
    /// ```lox
    /// if (cond) statement
    ///
    /// if (cond) {
    ///     ...
    /// }
    ///
    /// if (cond) {
    ///     ...
    /// } else {
    ///     ...
    /// }
    ///
    /// if (cond) {
    ///     ...
    /// } else if {
    ///     ...
    /// } else {
    ///     ...
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// - If a [LeftParen](TokenType) is not encountered
    /// - If a [RightParen](TokenType) is not encountered after processing the condition expression
    fn if_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume(TokenType::LeftParen, "expect '(' after if condition")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "expect ')' after if condition")?;

        let then_branch = self.statement()?;
        let mut else_branch: Option<Box<Statement>> = None;
        if self.matches_token(vec![TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Statement::If(cond, Box::new(then_branch), else_branch))
    }

    /// Parses a print statement.
    ///
    /// # Errors
    ///
    /// If a [Semicolon](TokenType) is not encountered after the expression; Only in strict mode
    fn print_statement(&mut self) -> Result<Statement, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "expected ';' after expression.")?;
        Ok(Statement::Print(expr))
    }

    /// Parses an expression statement.
    ///
    /// # Errors
    ///
    /// If a [Semicolon](TokenType) is not encountered after the expression; Only in strict mode
    fn expr_statement(&mut self) -> Result<Statement, ParserError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "expected ';' after expression.")?;
        Ok(Statement::Expr(expr))
    }

    /// Produces an expression.
    ///
    /// This is a one liner at the moments since expressions are first treated as an assignment and
    /// drilled down all the way to the lowest precedence production. See the module documentation
    /// for a list of the productions.
    fn expression(&mut self) -> Result<Expression, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression, ParserError> {
        let expr = self.logical_or()?;

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

    fn logical_or(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.logical_and()?;

        while self.matches_token(vec![TokenType::Or]) {
            let op = self.previous();
            let right = self.logical_and()?;
            expr = Expression::Logical(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.equality()?;

        while self.matches_token(vec![TokenType::And]) {
            let op = self.previous();
            let right = self.equality()?;
            expr = Expression::Logical(Box::new(expr), op, Box::new(right));
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
            self.consume(TokenType::RightParen, "expect ')' after expression.")?;
            expr = expr.group(group);
        } else if self.matches_token(vec![TokenType::Identifier]) {
            expr = expr.variable(self.previous());
        } else {
            eprintln!("{:#?}", self.peek());
            return Err(ParserError {
                cause: "expect expression".into(),
            });
        }

        expr.build().map_err(|e| ParserError {
            cause: e.to_string(),
        })
    }

    /// Advances the current position of the parser in the symbol list. Returns the consumed token
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.curr += 1;
        }

        self.previous()
    }

    /// Returns the previous symbol in the list.
    fn previous(&self) -> Token {
        self.tokens[self.curr - 1].clone()
    }

    /// Checks if the current symbol matches a list of symbol types; if it matches the parser
    /// advances one symbol ahead.
    fn matches_token(&mut self, tokens: Vec<TokenType>) -> bool {
        for token_type in tokens.iter() {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Checks if the current symbol matches a specific type
    fn check(&self, _type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == *_type
    }

    /// Checks if the parser has reached the end of the symbol list
    fn is_at_end(&self) -> bool {
        if !self.tokens.is_empty() {
            self.peek().token_type == TokenType::Eof
        } else {
            true
        }
    }

    /// Peeks one item ahead of the current symbol list position
    fn peek(&self) -> &Token {
        &self.tokens[self.curr]
    }

    /// Advances the parser if the specified symbol is matched otherwise it raises a parse error.
    ///
    /// # Errors
    ///
    /// If `_type` does not match the symbol at the `self.curr` position in the list.
    fn consume(&mut self, _type: TokenType, msg: &str) -> Result<Token, ParserError> {
        if self.check(&_type) {
            return Ok(self.advance());
        }

        if !self.strict && _type == TokenType::Semicolon {
            Ok(TokenBuilder::default().build())
        } else {
            Err(ParserError {
                cause: msg.to_string(),
            })
        }
    }

    /// Synchronizes the parser after an error is encountered.
    ///
    /// This function discards symbols within a context until a new context is encountered. It
    /// helps ensure that the parser continues to check for other parsing errors within the code
    /// passed but avoids any errors that are caused due to the encountered parsing error.
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
}

#[cfg(test)]
mod test {
    use crate::scanner::Scanner;

    use super::*;

    #[test]
    fn syntax_tree_parsed_correctly() {
        let source = "(5 * 2) + 1 - 2;".to_string();
        let scanner = Scanner::new(source);
        let sink = io::sink();
        let mut parser = Parser::new(scanner.run().unwrap(), sink, true);
        let stmt = parser.parse();

        assert_eq!(1, stmt.len());
    }
}
