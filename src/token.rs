use std::fmt::Display;

use crate::token_type::TokenType;
use crate::LocationInfo;

/// Valid word in the language grammar
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub loc: LocationInfo,
}

/// Literal representation of a number
#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Boolean(bool),
    String(Vec<char>),
    None,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v.iter().collect::<String>()),
            Self::Boolean(v) => write!(f, "{}", v),
            Self::None => write!(f, ""),
        }
    }
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Number(num) => {
                if let Self::Number(other_n) = other {
                    num == other_n
                } else {
                    false
                }
            }
            Self::Boolean(b) => {
                if let Self::Boolean(other_b) = other {
                    b == other_b
                } else {
                    false
                }
            }
            Self::String(string) => {
                if let Self::String(other_s) = other {
                    string.iter().zip(other_s).filter(|&(a, b)| a == b).count() == string.len()
                } else {
                    false
                }
            }
            Self::None => other == &Self::None,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.loc.eq(&other.loc)
            && self.lexeme == other.lexeme
            && self.token_type == other.token_type
    }
}

/// QOL Token interface; I suggest you use this if you ever
/// want to build tokens...
pub struct TokenBuilder {
    token: Token,
}

impl TokenBuilder {
    pub fn default() -> Self {
        TokenBuilder {
            token: Token {
                token_type: TokenType::Identifier,
                lexeme: String::new(),
                literal: Literal::None,
                loc: LocationInfo {
                    column: 0,
                    line: 0,
                    len: 0,
                },
            },
        }
    }

    pub fn current_lexeme(&self) -> &str {
        self.token.lexeme.as_str()
    }

    pub fn append_lexeme(mut self, character: char) -> Self {
        let token = &mut self.token;
        token.lexeme.push(character);
        token.loc.len += 1;

        self
    }

    pub fn location(mut self, col: usize, line: usize) -> Self {
        let token = &mut self.token;
        token.loc.column = col;
        token.loc.line = line;

        self
    }

    pub fn token_type(mut self, token_type: TokenType) -> Self {
        let token = &mut self.token;
        token.token_type = token_type;

        self
    }

    pub fn build(mut self) -> Token {
        match self.token.token_type {
            TokenType::True | TokenType::False => {
                self.token.literal = Literal::Boolean(self.token.token_type == TokenType::True)
            }
            TokenType::Number => {
                self.token.literal = Literal::Number(self.current_lexeme().parse().unwrap())
            }
            TokenType::Eof => {}
            _ => {
                self.token.literal =
                    Literal::String(self.current_lexeme().chars().collect::<Vec<char>>())
            }
        }
        self.token
    }
}
