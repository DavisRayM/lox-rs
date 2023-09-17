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
    NewLine,
    Eof,
    Tab,
    CarriageReturn,
    Space,
    Identifier,
    String,
    Number,
    Not,
    NotEqual,
    Equal,
    EqualEqual,
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

impl TryFrom<char> for TokenType {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            ')' => Ok(TokenType::RightParen),
            '(' => Ok(TokenType::LeftParen),
            '}' => Ok(TokenType::RightBrace),
            '{' => Ok(TokenType::LeftBrace),
            ',' => Ok(TokenType::Comma),
            '.' => Ok(TokenType::Dot),
            '-' => Ok(TokenType::Minus),
            '+' => Ok(TokenType::Plus),
            ';' => Ok(TokenType::SemiColon),
            '/' => Ok(TokenType::Slash),
            '*' => Ok(TokenType::Star),
            '<' => Ok(TokenType::Less),
            '>' => Ok(TokenType::Greater),
            '!' => Ok(TokenType::Not),
            '=' => Ok(TokenType::Equal),
            '\n' => Ok(TokenType::NewLine),
            '\t' => Ok(TokenType::Tab),
            '\r' => Ok(TokenType::CarriageReturn),
            ' ' => Ok(TokenType::Space),
            _ => Ok(TokenType::Identifier),
        }
    }
}

/// Token identified during lexical analysis
#[derive(Debug, Clone)]
pub struct Token {
    pub _type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
pub enum Literal {
    String(String),
    Number(i32),
    Boolean(bool),
    Generic(String),
}

impl Token {
    pub fn literal<T: Sized + 'static>(&self) -> Literal {
        match self._type {
            TokenType::String => Literal::String(self.lexeme.clone()),
            TokenType::Number => Literal::Number(self.lexeme.parse::<i32>().unwrap()),
            TokenType::True => Literal::Boolean(true),
            TokenType::False => Literal::Boolean(false),
            _ => Literal::Generic(self.lexeme.clone()),
        }
    }
}
