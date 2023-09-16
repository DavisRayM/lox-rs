use crate::tokens::{Token, TokenType};

#[derive(Clone, Debug)]
pub enum Expression {
    // unary -> token expr
    Unary(Token, Box<Expression>),
    // binary -> expr token expr
    Binary(Box<Expression>, Token, Box<Expression>),
    // groupint -> expr
    Grouping(Box<Expression>),
    // literal -> token
    Literal(Token),
}

#[derive(Clone, Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    curr: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, curr: 0 }
    }

    pub fn process_expression(&mut self) -> Expression {
        self.process_equality()
    }

    fn consume(&mut self) {
        self.curr += 1;
    }

    fn process_primary(&mut self) -> Expression {
        let token = self.tokens[self.curr].clone();

        match token._type {
            TokenType::False | TokenType::True | TokenType::Number | TokenType::String => {
                self.curr += 1;
                Expression::Literal(token)
            }
            TokenType::LeftParen => {
                self.curr += 1;
                let expr = self.process_expression();
                self.consume();
                Expression::Grouping(Box::new(expr))
            }
            _ => {
                self.curr += 1;
                self.process_expression()
            }
        }
    }

    fn process_unary(&mut self) -> Expression {
        match self.tokens[self.curr]._type {
            TokenType::Not => Expression::Unary(
                self.tokens[self.curr].clone(),
                Box::new(self.process_unary()),
            ),
            TokenType::Print => {
                let token = self.tokens[self.curr].clone();
                self.curr += 1;
                Expression::Unary(token, Box::new(self.process_unary()))
            }
            _ => self.process_primary(),
        }
    }

    fn process_factor(&mut self) -> Expression {
        let mut expr = self.process_unary();

        while self.tokens[self.curr]._type == TokenType::Slash
            || self.tokens[self.curr]._type == TokenType::Star
        {
            expr = Expression::Binary(
                Box::new(expr),
                self.tokens[self.curr].clone(),
                Box::new(self.process_unary()),
            )
        }

        expr
    }

    fn process_term(&mut self) -> Expression {
        let mut expr = self.process_factor();
        while self.tokens[self.curr]._type == TokenType::Minus
            || self.tokens[self.curr]._type == TokenType::Plus
        {
            expr = Expression::Binary(
                Box::new(expr),
                self.tokens[self.curr].clone(),
                Box::new(self.process_factor()),
            );
        }

        expr
    }

    fn process_comparison(&mut self) -> Expression {
        let mut expr = self.process_term();

        while self.tokens[self.curr]._type == TokenType::Greater
            || self.tokens[self.curr]._type == TokenType::GreaterEqual
            || self.tokens[self.curr]._type == TokenType::Less
            || self.tokens[self.curr]._type == TokenType::LessEqual
        {
            expr = Expression::Binary(
                Box::new(expr),
                self.tokens[self.curr].clone(),
                Box::new(self.process_term()),
            );
        }

        expr
    }

    fn process_equality(&mut self) -> Expression {
        let mut expr = self.process_comparison();

        while self.tokens[self.curr]._type == TokenType::NotEqual {
            expr = Expression::Binary(
                Box::new(expr),
                self.tokens[self.curr].clone(),
                Box::new(self.process_comparison()),
            );
        }

        expr
    }
}
