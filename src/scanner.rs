use crate::{
    errors::ScanError,
    tokens::{Token, TokenType},
};

const SINGLE_CHAR_TOKENS: [TokenType; 11] = [
    TokenType::RightParen,
    TokenType::LeftParen,
    TokenType::RightBrace,
    TokenType::LeftBrace,
    TokenType::Comma,
    TokenType::Dot,
    TokenType::Minus,
    TokenType::SemiColon,
    TokenType::Slash,
    TokenType::Plus,
    TokenType::Star,
];

const FORMATTING_TOKENS: [TokenType; 4] = [
    TokenType::NewLine,
    TokenType::Tab,
    TokenType::CarriageReturn,
    TokenType::Space,
];

const LOGICAL_TOKENS: [TokenType; 4] = [
    TokenType::Not,
    TokenType::Less,
    TokenType::Greater,
    TokenType::Equal,
];

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
    next: usize,
    current_col: usize,
    current_row: usize,
}

pub type ScannerResult<T> = Result<T, ScanError>;

impl Scanner {
    pub fn new(source: String) -> ScannerResult<Self> {
        let mut scanner = Self {
            tokens: Vec::new(),
            source: source.chars().collect(),
            next: 0,
            current_row: 1,
            current_col: 1,
        };

        if let Err(e) = scanner.scan_tokens() {
            return Err(ScanError {
                line: scanner.current_row,
                column: scanner.current_col,
                msg: e,
            });
        }

        Ok(scanner)
    }

    fn peek_next(&self) -> Option<TokenType> {
        let next_pos = self.next + 1;

        if next_pos >= self.source.len() {
            return None;
        }

        let token_type = TokenType::try_from(self.source[next_pos]);
        match token_type {
            Ok(token_type) => Some(token_type),
            Err(_) => None,
        }
    }

    fn scan_tokens(&mut self) -> Result<(), String> {
        while self.next < self.source.len() {
            let mut lexeme: Vec<char> = vec![self.source[self.next]];
            let mut token_type: TokenType = match TokenType::try_from(lexeme[0]) {
                Ok(token_type) => token_type,
                Err(e) => {
                    return Err(e);
                }
            };
            let mut is_new_line = false;
            let initial_pos = self.next;

            if SINGLE_CHAR_TOKENS
                .to_vec()
                .iter()
                .any(|_type| *_type == token_type)
            {
                self.add_token(token_type, lexeme.iter().collect::<String>());
            } else if FORMATTING_TOKENS
                .to_vec()
                .iter()
                .any(|_type| *_type == token_type)
            {
                if token_type == TokenType::NewLine {
                    is_new_line = true;
                }
            } else if LOGICAL_TOKENS
                .to_vec()
                .iter()
                .any(|_type| *_type == token_type)
            {
                if self.peek_next().is_some() && self.peek_next().unwrap() == TokenType::Equal {
                    token_type = match token_type {
                        TokenType::Not => TokenType::NotEqual,
                        TokenType::Less => TokenType::LessEqual,
                        TokenType::Greater => TokenType::GreaterEqual,
                        TokenType::Equal => TokenType::EqualEqual,
                        _ => return Err("fatal error: unknown token".into()),
                    };
                    lexeme.push(self.source[self.next + 1]);
                    self.next += 1;
                }
                self.add_token(token_type, lexeme.iter().collect::<String>());
            } else {
                if token_type != TokenType::Identifier {
                    return Err("something went horribly wrong!".into());
                }

                token_type = match self.consume_indentifier(&mut lexeme) {
                    Ok(token_type) => token_type,
                    Err(e) => return Err(e),
                };
                self.add_token(token_type, lexeme.iter().collect::<String>());
            }

            if is_new_line {
                self.current_row += 1;
                self.current_col = 0;
            }
            self.next += 1;
            self.current_col += self.next - initial_pos;
        }
        Ok(())
    }

    fn consume_indentifier(&mut self, lexeme: &mut Vec<char>) -> Result<TokenType, String> {
        let char_rep = lexeme[0];
        if char_rep == '"' {
            lexeme.clear();
            loop {
                let next_pos = self.next + 1;

                if next_pos >= self.source.len() {
                    return Err("unclosed string missing closing `\"`".into());
                }

                if self.source[next_pos] == '"' {
                    self.next += 1;
                    break;
                }

                lexeme.push(self.source[next_pos]);
                self.next += 1;
            }

            Ok(TokenType::String)
        } else if Self::is_digit(char_rep) {
            loop {
                let next_pos = self.next + 1;

                if self.peek_next().is_none() || !Self::is_numeric(self.source[next_pos]) {
                    break;
                } else {
                    lexeme.push(self.source[next_pos]);
                    self.next += 1;
                }
            }

            Ok(TokenType::Number)
        } else if Self::is_alphabetic(char_rep) {
            loop {
                let next_pos = self.next + 1;

                if self.peek_next().is_none() {
                    break;
                } else if Self::is_alphanumeric(self.source[next_pos]) {
                    lexeme.push(self.source[next_pos]);
                    self.next += 1;
                } else {
                    break;
                }
            }

            Ok(Self::process_identifier(&lexeme.iter().collect::<String>()))
        } else {
            return Err("unknown character".into());
        }
    }

    fn add_token(&mut self, _type: TokenType, lexeme: String) {
        let token = Token {
            _type,
            lexeme,
            line: self.current_row,
            column: self.current_col,
        };
        self.tokens.push(token);
    }

    fn process_identifier(identifier: &str) -> TokenType {
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
            (TokenType::LeftParen, "(".to_string(), 1, 1),
            (TokenType::RightParen, ")".to_string(), 1, 2),
            (TokenType::LeftBrace, "{".to_string(), 1, 3),
            (TokenType::RightBrace, "}".to_string(), 1, 4),
            (TokenType::Comma, ",".to_string(), 1, 5),
            (TokenType::Dot, ".".to_string(), 1, 6),
            (TokenType::Minus, "-".to_string(), 1, 7),
            (TokenType::Plus, "+".to_string(), 1, 8),
            (TokenType::SemiColon, ";".to_string(), 1, 9),
            (TokenType::Slash, "/".to_string(), 1, 10),
            (TokenType::Star, "*".to_string(), 1, 12),
        ];
        assert_expected_tokens(scanner, expected);
    }

    #[test]
    fn captures_string_and_number_tokens() {
        let content = "\"Hey there 2\" 25 12.32";
        let scanner = Scanner::new(content.into()).unwrap();

        let expected = vec![
            (TokenType::String, "Hey there 2".to_string(), 1, 1),
            (TokenType::Number, "25".to_string(), 1, 15),
            (TokenType::Number, "12.32".to_string(), 1, 18),
        ];
        assert_expected_tokens(scanner, expected);
    }

    #[test]
    fn captures_two_character_tokens() {
        let content = "<=<>=>";
        let scanner = Scanner::new(content.into()).unwrap();

        let eexpected = vec![
            (TokenType::LessEqual, "<=".to_string(), 1, 1),
            (TokenType::Less, "<".to_string(), 1, 3),
            (TokenType::GreaterEqual, ">=".to_string(), 1, 4),
            (TokenType::Greater, ">".to_string(), 1, 6),
        ];
        assert_expected_tokens(scanner, eexpected);
    }

    #[test]
    fn captures_identifiers_accurately() {
        let content = "and class else false for if or print return super true let while some_identifier someIdentifier identifier32";
        let scanner = Scanner::new(content.into()).unwrap();

        let expected = vec![
            (TokenType::And, "and".to_string(), 1, 1),
            (TokenType::Class, "class".to_string(), 1, 5),
            (TokenType::Else, "else".to_string(), 1, 11),
            (TokenType::False, "false".to_string(), 1, 16),
            (TokenType::For, "for".to_string(), 1, 22),
            (TokenType::If, "if".to_string(), 1, 26),
            (TokenType::Or, "or".to_string(), 1, 29),
            (TokenType::Print, "print".to_string(), 1, 32),
            (TokenType::Return, "return".to_string(), 1, 38),
            (TokenType::Super, "super".to_string(), 1, 45),
            (TokenType::True, "true".to_string(), 1, 51),
            (TokenType::Let, "let".to_string(), 1, 56),
            (TokenType::While, "while".to_string(), 1, 60),
            (TokenType::Identifier, "some_identifier".to_string(), 1, 66),
            (TokenType::Identifier, "someIdentifier".to_string(), 1, 82),
            (TokenType::Identifier, "identifier32".to_string(), 1, 97),
        ];
        assert_expected_tokens(scanner, expected);
    }

    #[test]
    fn captures_content_successfully() {
        let content = "let num = 23;\nprint(num);";
        let scanner = Scanner::new(content.into()).unwrap();

        let expected = vec![
            (TokenType::Let, "let".to_string(), 1, 1),
            (TokenType::Identifier, "num".to_string(), 1, 5),
            (TokenType::Equal, "=".to_string(), 1, 9),
            (TokenType::Number, "23".to_string(), 1, 11),
            (TokenType::SemiColon, ";".to_string(), 1, 13),
            (TokenType::Print, "print".to_string(), 2, 1),
            (TokenType::LeftParen, "(".to_string(), 2, 6),
            (TokenType::Identifier, "num".to_string(), 2, 7),
            (TokenType::RightParen, ")".to_string(), 2, 10),
            (TokenType::SemiColon, ";".to_string(), 2, 11),
        ];
        assert_expected_tokens(scanner, expected);
    }
}
