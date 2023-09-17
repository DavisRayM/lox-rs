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
use crate::tokens::{Token, TokenType};

#[derive(Clone, Debug)]
pub enum Expression {
    Unary(Token, Box<Expression>),
    Binary(Box<Expression>, Token, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(String),
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
                format!("({} {} {})", token.lexeme, expr, r_expr)
            }
            Expression::Grouping(expr) => {
                let expr: String = expr.as_ref().to_owned().into();
                format!("(group {})", expr)
            }
            Expression::Literal(token) => token,
        }
    }
}

/// AST Parser for the Lox language
pub struct Parser {
    current: usize,
    source: Vec<Token>,
}

impl Parser {
    pub fn new(source: Vec<Token>) -> Self {
        Self { source, current: 0 }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, String> {
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

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;

        while self.advance_if_match(vec![TokenType::NotEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let r_expr = self.parse_comparison()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(r_expr));
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_term()?;

        while self.advance_if_match(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let rexpr = self.parse_term()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(rexpr));
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_factor()?;

        while self.advance_if_match(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let rexpr = self.parse_factor()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(rexpr));
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;

        while self.advance_if_match(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let rexpr = self.parse_unary()?;
            expr = Expression::Binary(Box::new(expr), operator, Box::new(rexpr));
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.advance_if_match(vec![TokenType::Not, TokenType::Minus]) {
            let operator = self.previous();
            let rexpr = self.parse_unary()?;
            Ok(Expression::Unary(operator, Box::new(rexpr)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
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
            Err("unexpected expression".into())
        }
    }

    fn check_and_consume(&mut self, token_type: TokenType) -> Result<(), String> {
        if self.source[self.current]._type != token_type {
            return Err(format!("expected '{:?}' after expression", token_type));
        }

        self.current += 1;
        Ok(())
    }
}
