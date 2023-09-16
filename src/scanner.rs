use crate::{
    errors::ScanError,
    tokens::{Token, TokenType},
};

/// Scanner is used for lexically analysis string content
///
/// The scanner performs lexical analysis on string content afterwhich it
/// provides access to the token read from the content. Panics if invalid
/// token is read.
///
/// # Panics
///
/// Panics if an invalid token is found.
pub struct Scanner {
    pub tokens: Vec<Token>,
    source: Vec<char>,
    current_pos: usize,
    current_col: usize,
    current_row: usize,
}

pub type ScannerResult<T> = Result<T, ScanError>;

impl Scanner {
    pub fn new(source: String) -> ScannerResult<Self> {
        let mut scanner = Self {
            tokens: Vec::new(),
            source: source.chars().collect(),
            current_pos: 0,
            current_row: 0,
            current_col: 0,
        };

        if let Err(e) = scanner.scan_tokens() {
            return Err(ScanError {
                line: scanner.current_row,
                column: scanner.current_pos,
                msg: e,
            });
        }

        Ok(scanner)
    }

    fn scan_tokens(&mut self) -> Result<(), String> {
        while (self.current_pos) < self.source.len() {
            match self.scan() {
                Ok(_) => {}
                Err(e) => return Err(e),
            };
        }
        Ok(())
    }

    fn add_token(&mut self, _type: TokenType, lexeme: String, literal: Option<String>) {
        let token = Token {
            _type,
            lexeme,
            literal,
            line: self.current_row,
            column: self.current_col,
        };
        self.tokens.push(token);
    }

    fn scan(&mut self) -> Result<(), String> {
        let start = self.source[self.current_pos];
        let mut is_newline = false;
        match start {
            ')' => self.add_token(TokenType::RightParen, start.into(), None),
            '(' => self.add_token(TokenType::LeftParen, start.into(), None),
            '}' => self.add_token(TokenType::RightBrace, start.into(), None),
            '{' => self.add_token(TokenType::LeftBrace, start.into(), None),
            ',' => self.add_token(TokenType::Comma, start.into(), None),
            '.' => self.add_token(TokenType::Dot, start.into(), None),
            '-' => self.add_token(TokenType::Minus, start.into(), None),
            ';' => self.add_token(TokenType::SemiColon, start.into(), None),
            '/' => self.add_token(TokenType::Slash, start.into(), None),
            '+' => self.add_token(TokenType::Plus, start.into(), None),
            '\n' => {
                is_newline = true;
            }
            '\t' => {}
            '\r' => {}
            ' ' => {}
            '*' => self.add_token(TokenType::Star, start.into(), None),
            '=' => self.add_token(TokenType::Equal, start.into(), None),
            '!' => {
                let next_pos = self.current_pos + 1;
                if next_pos < self.source.len() && self.source[next_pos] == '=' {
                    let lexeme: String = self.capture_lexeme(next_pos + 1);
                    self.add_token(TokenType::NotEqual, lexeme, None);
                    self.current_pos += 1;
                    self.current_col += 1;
                } else {
                    self.add_token(TokenType::Not, start.into(), None);
                }
            }
            '<' => {
                let next_pos = self.current_pos + 1;
                if next_pos < self.source.len() && self.source[next_pos] == '=' {
                    let lexeme: String = self.capture_lexeme(next_pos + 1);
                    self.add_token(TokenType::LessEqual, lexeme, None);
                    self.current_pos += 1;
                    self.current_col += 1;
                } else {
                    self.add_token(TokenType::Less, start.into(), None);
                }
            }
            '>' => {
                let next_pos = self.current_pos + 1;
                if next_pos < self.source.len() && self.source[next_pos] == '=' {
                    let lexeme: String = self.capture_lexeme(next_pos + 1);
                    self.add_token(TokenType::GreaterEqual, lexeme, None);
                    self.current_pos += 1;
                    self.current_col += 1;
                } else {
                    self.add_token(TokenType::Greater, start.into(), None);
                }
            }
            '"' => {
                let pos = self.current_pos;

                let start_pos = pos + 1;
                let mut end_pos = start_pos;

                loop {
                    if end_pos >= self.source.len() {
                        return Err("unclosed string; missing closing `\"`".into());
                    }

                    if self.source[end_pos] == '"' {
                        break;
                    }
                    end_pos += 1;
                }

                self.current_pos = start_pos;
                let lexeme: String = self.capture_lexeme(end_pos);

                self.current_pos = pos;
                self.add_token(TokenType::String, lexeme.clone(), Some(lexeme));
                self.current_pos = end_pos;
                self.current_col += end_pos - pos;
            }
            _ => {
                let start_pos = self.current_pos;

                if Self::is_digit(self.source[start_pos]) {
                    self.add_digit(start_pos);
                } else if Self::is_alphabetic(self.source[start_pos]) {
                    self.add_identifier(start_pos);
                } else {
                    return Err(format!("unexpected character `{}`", self.source[start_pos]));
                }
            }
        };

        self.current_pos += 1;
        if is_newline {
            self.current_row += 1;
            self.current_col = 0;
        } else {
            self.current_col += 1;
        }

        Ok(())
    }

    fn add_identifier(&mut self, start_pos: usize) -> usize {
        let mut lexeme: Vec<char> = Vec::new();
        lexeme.push(self.source[start_pos]);

        let mut curr_pos = start_pos;
        if curr_pos < self.source.len() {
            while (curr_pos + 1) < self.source.len()
                && Self::is_alphanumeric(self.source[curr_pos + 1])
            {
                curr_pos += 1;
                lexeme.push(self.source[curr_pos]);
            }
        }

        let lexeme = lexeme.iter().collect::<String>();
        let t = self.process_identifier(&lexeme);
        self.add_token(t, lexeme.clone(), Some(lexeme));
        self.current_pos = curr_pos;
        self.current_col += curr_pos - start_pos;
        curr_pos
    }

    fn add_digit(&mut self, start_pos: usize) -> usize {
        let mut lexeme: Vec<char> = Vec::new();

        lexeme.push(self.source[start_pos]);
        let mut curr_pos = start_pos;
        if curr_pos < self.source.len() {
            while (curr_pos + 1) < self.source.len() && Self::is_numeric(self.source[curr_pos + 1])
            {
                curr_pos += 1;
                lexeme.push(self.source[curr_pos]);
            }
        }

        let lexeme = lexeme.iter().collect::<String>();
        self.add_token(TokenType::Number, lexeme.clone(), Some(lexeme));
        self.current_pos = curr_pos;
        self.current_col += curr_pos - start_pos;
        curr_pos
    }

    fn process_identifier(&mut self, identifier: &str) -> TokenType {
        match identifier {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "true" => TokenType::True,
            "let" => TokenType::Let,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        }
    }

    fn is_digit(c: char) -> bool {
        c.is_numeric()
    }

    fn is_numeric(c: char) -> bool {
        c == '.' || c.is_numeric()
    }

    fn is_alphabetic(c: char) -> bool {
        c.is_alphabetic()
    }

    fn is_alphanumeric(c: char) -> bool {
        c != ' ' && (c.is_alphanumeric() || c == '_')
    }

    fn capture_lexeme(&self, end: usize) -> String {
        self.source[self.current_pos..end].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_expected_tokens(scanner: Scanner, expected: Vec<(TokenType, String, usize, usize)>) {
        assert_eq!(
            scanner.tokens.len(),
            expected.len(),
            "{:#?}",
            scanner.tokens
        );
        for (idx, token) in scanner.tokens.iter().enumerate() {
            assert_eq!(token._type, expected[idx].0);
            assert_eq!(token.lexeme, expected[idx].1);
            assert_eq!(token.line, expected[idx].2);
            assert_eq!(token.column, expected[idx].3);
        }
    }

    #[test]
    fn captures_single_character_tokens() {
        let content = "(){},.-+;/ *";
        let scanner = Scanner::new(content.into()).unwrap();

        let expected = vec![
            (TokenType::LeftParen, "(".to_string(), 0, 0),
            (TokenType::RightParen, ")".to_string(), 0, 1),
            (TokenType::LeftBrace, "{".to_string(), 0, 2),
            (TokenType::RightBrace, "}".to_string(), 0, 3),
            (TokenType::Comma, ",".to_string(), 0, 4),
            (TokenType::Dot, ".".to_string(), 0, 5),
            (TokenType::Minus, "-".to_string(), 0, 6),
            (TokenType::Plus, "+".to_string(), 0, 7),
            (TokenType::SemiColon, ";".to_string(), 0, 8),
            (TokenType::Slash, "/".to_string(), 0, 9),
            (TokenType::Star, "*".to_string(), 0, 11),
        ];
        assert_expected_tokens(scanner, expected);
    }

    #[test]
    fn captures_string_and_number_tokens() {
        let content = "\"Hey there 2\" 25 12.32";
        let scanner = Scanner::new(content.into()).unwrap();

        let expected = vec![
            (TokenType::String, "Hey there 2".to_string(), 0, 0),
            (TokenType::Number, "25".to_string(), 0, 14),
            (TokenType::Number, "12.32".to_string(), 0, 17),
        ];
        assert_expected_tokens(scanner, expected);
    }

    #[test]
    fn captures_two_character_tokens() {
        let content = "<=<>=>";
        let scanner = Scanner::new(content.into()).unwrap();

        let eexpected = vec![
            (TokenType::LessEqual, "<=".to_string(), 0, 0),
            (TokenType::Less, "<".to_string(), 0, 2),
            (TokenType::GreaterEqual, ">=".to_string(), 0, 3),
            (TokenType::Greater, ">".to_string(), 0, 5),
        ];
        assert_expected_tokens(scanner, eexpected);
    }

    #[test]
    fn captures_identifiers_accurately() {
        let content = "and class else false for if or print return super true let while some_identifier someIdentifier identifier32";
        let scanner = Scanner::new(content.into()).unwrap();

        let expected = vec![
            (TokenType::And, "and".to_string(), 0, 0),
            (TokenType::Class, "class".to_string(), 0, 4),
            (TokenType::Else, "else".to_string(), 0, 10),
            (TokenType::False, "false".to_string(), 0, 15),
            (TokenType::For, "for".to_string(), 0, 21),
            (TokenType::If, "if".to_string(), 0, 25),
            (TokenType::Or, "or".to_string(), 0, 28),
            (TokenType::Print, "print".to_string(), 0, 31),
            (TokenType::Return, "return".to_string(), 0, 37),
            (TokenType::Super, "super".to_string(), 0, 44),
            (TokenType::True, "true".to_string(), 0, 50),
            (TokenType::Let, "let".to_string(), 0, 55),
            (TokenType::While, "while".to_string(), 0, 59),
            (TokenType::Identifier, "some_identifier".to_string(), 0, 65),
            (TokenType::Identifier, "someIdentifier".to_string(), 0, 81),
            (TokenType::Identifier, "identifier32".to_string(), 0, 96),
        ];
        assert_expected_tokens(scanner, expected);
    }

    #[test]
    fn captures_content_successfully() {
        let content = "let num = 23;\nprint(num);";
        let scanner = Scanner::new(content.into()).unwrap();

        let expected = vec![
            (TokenType::Let, "let".to_string(), 0, 0),
            (TokenType::Identifier, "num".to_string(), 0, 4),
            (TokenType::Equal, "=".to_string(), 0, 8),
            (TokenType::Number, "23".to_string(), 0, 10),
            (TokenType::SemiColon, ";".to_string(), 0, 12),
            (TokenType::Print, "print".to_string(), 1, 0),
            (TokenType::LeftParen, "(".to_string(), 1, 5),
            (TokenType::Identifier, "num".to_string(), 1, 6),
            (TokenType::RightParen, ")".to_string(), 1, 9),
            (TokenType::SemiColon, ";".to_string(), 1, 10),
        ];
        assert_expected_tokens(scanner, expected);
    }
}
