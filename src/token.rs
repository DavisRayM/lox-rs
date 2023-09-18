use core::fmt;

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
    And,
    Or,

    // Keywords
    Class,
    Else,
    False,
    For,
    If,
    Print,
    Return,
    Super,
    True,
    Let,
    While,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_rep = match self {
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::LeftBrace => "{",
            TokenType::RightBrace => "}",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::Minus => "-",
            TokenType::Plus => "+",
            TokenType::SemiColon => ";",
            TokenType::Slash => "/",
            TokenType::Star => "*",
            TokenType::NewLine => "new line",
            TokenType::Eof => "end of file",
            TokenType::Tab => "tab",
            TokenType::CarriageReturn => "return",
            TokenType::Space => "space",
            TokenType::Identifier => "identifier",
            TokenType::String => "string",
            TokenType::Number => "number",
            TokenType::Not => "!",
            TokenType::NotEqual => "!=",
            TokenType::Equal => "=",
            TokenType::EqualEqual => "==",
            TokenType::Less => "<",
            TokenType::LessEqual => "<=",
            TokenType::Greater => ">",
            TokenType::GreaterEqual => ">=",
            TokenType::And => "&&",
            TokenType::Class => "class",
            TokenType::Else => "else",
            TokenType::False => "false",
            TokenType::For => "for",
            TokenType::If => "if",
            TokenType::Or => "||",
            TokenType::Print => "print",
            TokenType::Return => "return",
            TokenType::Super => "super",
            TokenType::True => "true",
            TokenType::Let => "let",
            TokenType::While => "while",
        };

        write!(f, "{}", str_rep)
    }
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
