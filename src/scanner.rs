//! Lexical Analyzer(Lexer)
use crate::{
    errors::ScannerError,
    token::{Token, TokenBuilder},
    token_type::TokenType,
    LocationInfo,
};

/// Lexical scanner/analyzer
///
/// Scanner reads through the `source` passed in and extracts `Token`s from the
/// code
pub struct Scanner {
    source: Vec<char>,
    pub tokens: Vec<Token>,
    pub loc: LocationInfo,
}

const IDENTIFIERS: [(&str, TokenType); 16] = [
    ("and", TokenType::And),
    ("class", TokenType::Class),
    ("else", TokenType::Else),
    ("false", TokenType::False),
    ("for", TokenType::For),
    ("if", TokenType::If),
    ("nil", TokenType::Nil),
    ("or", TokenType::Or),
    ("print", TokenType::Print),
    ("return", TokenType::Return),
    ("super", TokenType::Super),
    ("this", TokenType::This),
    ("true", TokenType::True),
    ("var", TokenType::Var),
    ("while", TokenType::While),
    ("break", TokenType::Break),
];

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            loc: LocationInfo {
                column: 0,
                line: 1,
                len: 0,
            },
        }
    }

    pub fn run(&mut self) -> Result<(), ScannerError> {
        if let Some(last) = self.tokens.last() {
            if last.token_type == TokenType::Eof {
                return Ok(());
            }
        }

        loop {
            // Terminate scanner if theres nothing else to scan
            if self.is_at_end() {
                self._add_token(
                    vec![],
                    TokenType::Eof,
                    TokenBuilder::default().location(self.loc.column, self.loc.line),
                );
                break Ok(());
            }

            self.scan_token()?;
        }
    }

    fn scan_token(&mut self) -> Result<(), ScannerError> {
        let builder = TokenBuilder::default().location(self.loc.column, self.loc.line);
        let ch = self.next();

        match ch {
            ' ' | '\r' | '\t' => (),
            '\n' => self.loc.line += 1,
            '(' => self._add_token([ch].to_vec(), TokenType::LeftParen, builder),
            ')' => self._add_token([ch].to_vec(), TokenType::RightParen, builder),
            '{' => self._add_token([ch].to_vec(), TokenType::LeftBrace, builder),
            '}' => self._add_token([ch].to_vec(), TokenType::RightBrace, builder),
            ',' => self._add_token([ch].to_vec(), TokenType::Comma, builder),
            '.' => self._add_token([ch].to_vec(), TokenType::Dot, builder),
            '-' => self._add_token([ch].to_vec(), TokenType::Minus, builder),
            '+' => self._add_token([ch].to_vec(), TokenType::Plus, builder),
            ';' => self._add_token([ch].to_vec(), TokenType::Semicolon, builder),
            '*' => self._add_token([ch].to_vec(), TokenType::Star, builder),
            '!' => {
                if let Some(extra_ch) = self.next_if(Box::new(|ch: char| ch == '=')) {
                    self._add_token([ch, extra_ch].to_vec(), TokenType::BangEqual, builder)
                } else {
                    self._add_token([ch].to_vec(), TokenType::Bang, builder)
                }
            }
            '=' => {
                if let Some(extra_ch) = self.next_if(Box::new(|ch: char| ch == '=')) {
                    self._add_token([ch, extra_ch].to_vec(), TokenType::EqualEqual, builder)
                } else {
                    self._add_token([ch].to_vec(), TokenType::Equal, builder)
                }
            }
            '<' => {
                if let Some(extra_ch) = self.next_if(Box::new(|ch: char| ch == '=')) {
                    self._add_token([ch, extra_ch].to_vec(), TokenType::LessEqual, builder)
                } else {
                    self._add_token([ch].to_vec(), TokenType::Less, builder)
                }
            }
            '>' => {
                if let Some(extra_ch) = self.next_if(Box::new(|ch: char| ch == '=')) {
                    self._add_token([ch, extra_ch].to_vec(), TokenType::GreaterEqual, builder)
                } else {
                    self._add_token([ch].to_vec(), TokenType::Greater, builder)
                }
            }
            '/' => {
                if self.next_if(Box::new(|ch: char| ch == '/')).is_some() {
                    // Discard the comment
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.next();
                    }
                } else {
                    self._add_token([ch].to_vec(), TokenType::Slash, builder)
                }
            }
            '"' => self._add_string(builder)?,
            ch => {
                // Check if character is a number or identifier
                // before raising an error
                if ch.is_ascii_digit() {
                    self._add_number(builder.append_lexeme(ch))?;
                } else if _is_alpha(ch) {
                    self._add_identifier(builder.append_lexeme(ch));
                } else {
                    return Err(ScannerError {
                        cause: format!("unexpected character: {}", ch),
                        location: builder.build().loc,
                    });
                }
            }
        };

        Ok(())
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source[self.loc.len]
    }

    fn peek_next(&self) -> char {
        if self.is_at_end() || self.loc.len + 1 >= self.source.len() {
            return '\0';
        }

        self.source[self.loc.len + 1]
    }

    fn next(&mut self) -> char {
        let ch = self.source[self.loc.len];

        self.loc.len += 1;
        self.loc.column += 1;

        ch
    }

    fn next_if(&mut self, func: Box<dyn Fn(char) -> bool>) -> Option<char> {
        if self.is_at_end() {
            return None;
        }

        let ch = self.source[self.loc.len];
        if func(ch) {
            Some(self.next())
        } else {
            None
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.loc.len == self.source.len()
    }

    fn _add_token(&mut self, chars: Vec<char>, token_type: TokenType, builder: TokenBuilder) {
        let mut builder = builder.token_type(token_type);

        for ch in chars {
            builder = builder.append_lexeme(ch);
        }

        self.tokens.push(builder.build());
    }

    fn _add_string(&mut self, builder: TokenBuilder) -> Result<(), ScannerError> {
        let mut builder = builder
            .token_type(TokenType::String)
            // Set location to the first character of the string
            // This felt more appropriate at the time but i might change
            // my mind in the future
            .location(self.loc.column, self.loc.line);

        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.loc.line += 1;
            }

            builder = builder.append_lexeme(self.next());
        }

        if self.is_at_end() {
            return Err(ScannerError {
                cause: "unterminated string".to_string(),
                location: builder.build().loc,
            });
        }

        self.next();
        self.tokens.push(builder.build());
        Ok(())
    }

    fn _add_number(&mut self, builder: TokenBuilder) -> Result<(), ScannerError> {
        let mut builder = builder.token_type(TokenType::Number);

        while self.peek().is_ascii_digit() {
            builder = builder.append_lexeme(self.next());
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            builder = builder.append_lexeme(self.next());

            while self.peek().is_ascii_digit() {
                builder = builder.append_lexeme(self.next());
            }
        }

        self.tokens.push(builder.build());

        Ok(())
    }

    fn _add_identifier(&mut self, builder: TokenBuilder) {
        let mut builder = builder.token_type(TokenType::Identifier);

        while !self.is_at_end() && (self.peek().is_ascii_alphanumeric() || self.peek() == '_') {
            builder = builder.append_lexeme(self.next());
        }

        let mut token_type = IDENTIFIERS
            .iter()
            .filter(|(s, _)| *s == builder.current_lexeme())
            .map(|(_, t)| t.clone())
            .collect::<Vec<TokenType>>();

        if token_type.len() == 1 {
            let token_type = token_type.pop().expect("expected a token type");
            builder = builder.token_type(token_type);
        }

        self.tokens.push(builder.build());
    }
}

fn _is_alpha(ch: char) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '_'
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::Literal;

    #[test]
    fn comments_are_discarded() {
        const SOURCE: &str = r#"// much comment such wow
// i also know how to comment
(( )){} // here are some tokens!"#;
        let mut s = Scanner::new(SOURCE.to_string());
        s.run().unwrap();

        assert_eq!(s.tokens.len(), 7);
        assert_eq!(
            s.loc,
            LocationInfo {
                column: SOURCE.len(),
                line: 3,
                len: SOURCE.len()
            }
        );
    }

    #[test]
    fn string_is_correctly_added() {
        const SOURCE: &str = "\"hello\" \"world!\"";
        let expected: [Token; 3] = [
            Token {
                token_type: TokenType::String,
                lexeme: "hello".to_string(),
                literal: Literal::String("hello".chars().collect::<Vec<char>>()),
                loc: LocationInfo {
                    column: 1,
                    line: 1,
                    len: 5,
                },
            },
            Token {
                token_type: TokenType::String,
                lexeme: "world!".to_string(),
                literal: Literal::String("world!".chars().collect::<Vec<char>>()),
                loc: LocationInfo {
                    column: 9,
                    line: 1,
                    len: 6,
                },
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                loc: LocationInfo {
                    column: 16,
                    line: 1,
                    len: 0,
                },
            },
        ];

        let mut s = Scanner::new(SOURCE.to_string());
        s.run().unwrap();

        assert_eq!(s.tokens, expected.to_vec());
    }

    #[test]
    fn identifiers_are_correctly_scanned() {
        const SOURCE: &str = "and super this some_var";
        let expected: [Token; 5] = [
            Token {
                token_type: TokenType::And,
                lexeme: "and".to_string(),
                literal: Literal::String("and".chars().collect::<Vec<char>>()),
                loc: LocationInfo {
                    column: 0,
                    line: 1,
                    len: 3,
                },
            },
            Token {
                token_type: TokenType::Super,
                lexeme: "super".to_string(),
                literal: Literal::String("super".chars().collect::<Vec<char>>()),
                loc: LocationInfo {
                    column: 4,
                    line: 1,
                    len: 5,
                },
            },
            Token {
                token_type: TokenType::This,
                lexeme: "this".to_string(),
                literal: Literal::String("super".chars().collect::<Vec<char>>()),
                loc: LocationInfo {
                    column: 10,
                    line: 1,
                    len: 4,
                },
            },
            Token {
                token_type: TokenType::Identifier,
                lexeme: "some_var".to_string(),
                literal: Literal::String("super".chars().collect::<Vec<char>>()),
                loc: LocationInfo {
                    column: 15,
                    line: 1,
                    len: 8,
                },
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                loc: LocationInfo {
                    column: 23,
                    line: 1,
                    len: 0,
                },
            },
        ];

        let mut s = Scanner::new(SOURCE.to_string());
        s.run().unwrap();

        assert_eq!(s.tokens, expected);
    }

    #[test]
    fn numbers_are_correctly_scanned() {
        const SOURCE: &str = "25 25.03 4343";
        let expected: [Token; 4] = [
            Token {
                token_type: TokenType::Number,
                lexeme: "25".to_string(),
                literal: Literal::Number(25_f64),
                loc: LocationInfo {
                    column: 0,
                    line: 1,
                    len: 2,
                },
            },
            Token {
                token_type: TokenType::Number,
                lexeme: "25.03".to_string(),
                literal: Literal::Number(25.03_f64),
                loc: LocationInfo {
                    column: 3,
                    line: 1,
                    len: 5,
                },
            },
            Token {
                token_type: TokenType::Number,
                lexeme: "4343".to_string(),
                literal: Literal::Number(4343_f64),
                loc: LocationInfo {
                    column: 9,
                    line: 1,
                    len: 4,
                },
            },
            Token {
                token_type: TokenType::Eof,
                lexeme: "".to_string(),
                literal: Literal::None,
                loc: LocationInfo {
                    column: 13,
                    line: 1,
                    len: 0,
                },
            },
        ];

        let mut s = Scanner::new(SOURCE.to_string());
        s.run().unwrap();

        assert_eq!(s.tokens, expected.to_vec());
    }

    #[test]
    #[should_panic]
    fn unterminated_string_is_caught() {
        const SOURCE: &str = "\"Hello worl";
        let mut s = Scanner::new(SOURCE.to_string());
        s.run().unwrap();
    }
}
