/// Type of a token
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,

    // One or two character tokens
    Identifier,
    String,
    Number,
    Not,
    NotEqual,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    If,
    Or,
    Print,
    Return,
    Super,
    True,
    Let,
    While,
}

/// Token identified during lexical analysis
#[derive(Debug, Clone)]
pub struct Token {
    pub _type: TokenType,
    pub lexeme: String,
    pub literal: Option<String>,
    pub line: u16,
}
