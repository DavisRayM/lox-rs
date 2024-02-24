#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
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
    Semicolon,
    Slash,
    Star,

    // Variable length tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literal tokens
    Identifier,
    String,
    Number,

    // Keyword tokens
    And,
    Break,
    Class,
    Else,
    False,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Character with no representation
    Eof,
}
