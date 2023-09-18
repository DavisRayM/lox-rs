use crate::{
    errors::ScanError,
    token::{Token, TokenType},
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

    fn next_type(&self) -> Option<TokenType> {
        if let Some(value) = self.peek_next() {
            let token_type = TokenType::try_from(value).unwrap();
            Some(token_type)
        } else {
            None
        }
    }

    fn scan_tokens(&mut self) -> Result<(), String> {
        while self.next < self.source.len() {
            let line = self.current_row;
            let col = self.current_col;

            let mut is_new_line = false;

            let mut lexeme: Vec<char> = vec![self.source[self.next]];
            let mut token_type: TokenType = match TokenType::try_from(lexeme[0]) {
                Ok(token_type) => token_type,
                Err(e) => {
                    return Err(e);
                }
            };

            if SINGLE_CHAR_TOKENS
                .to_vec()
                .iter()
                .any(|_type| *_type == token_type)
            {
                self.next();
                self.add_token(token_type, lexeme.iter().collect::<String>(), line, col);
            } else if FORMATTING_TOKENS
                .to_vec()
                .iter()
                .any(|_type| *_type == token_type)
            {
                self.next();
                if token_type == TokenType::NewLine {
                    is_new_line = true;
                }
            } else {
                self.next();
                token_type = match self.read_next_token(&mut lexeme) {
                    Ok(token_type) => token_type,
                    Err(e) => return Err(e),
                };
                self.add_token(token_type, lexeme.iter().collect::<String>(), line, col);
            }

            if is_new_line {
                self.current_row += 1;
                self.current_col = 1;
            }
        }
        Ok(())
    }

    fn next_matches(&self, s: char) -> bool {
        if let Some(value) = self.peek_next() {
            value == s
        } else {
            false
        }
    }

    fn has_next(&self) -> bool {
        self.source.len() > self.next
    }

    fn peek_next(&self) -> Option<char> {
        if !self.has_next() {
            None
        } else {
            Some(self.source[self.next])
        }
    }

    fn next(&mut self) -> Option<char> {
        if let Some(value) = self.peek_next() {
            self.next += 1;
            self.current_col += 1;
            Some(value)
        } else {
            None
        }
    }

    fn consume_until(&mut self, buf: &mut Vec<char>, until: char) -> Result<(), String> {
        loop {
            if self.next_matches(until) {
                self.next();
                break;
            } else {
                match self.next() {
                    Some(val) => buf.push(val),
                    None => {
                        return Err(format!("missing `{}`", until));
                    }
                }
            }
        }

        Ok(())
    }

    fn read_next_token(&mut self, lexeme: &mut Vec<char>) -> Result<TokenType, String> {
        let char_rep = lexeme[0];
        let token_type: TokenType;

        match char_rep {
            '"' => {
                token_type = TokenType::String;
                lexeme.clear();

                if let Err(e) = self.consume_until(lexeme, char_rep) {
                    Err(format!("unclosed {} {}", TokenType::String, e))
                } else {
                    Ok(token_type)
                }
            }
            '|' => {
                if self.next_matches(char_rep) {
                    lexeme.push(self.next().unwrap());
                    Ok(TokenType::Or)
                } else {
                    Err("unknown character".into())
                }
            }
            '&' => {
                if self.next_matches(char_rep) {
                    lexeme.push(self.next().unwrap());
                    Ok(TokenType::And)
                } else {
                    Err("unknown character".into())
                }
            }
            '<' => {
                if self.next_matches('=') {
                    lexeme.push(self.next().unwrap());
                    Ok(TokenType::LessEqual)
                } else {
                    Ok(TokenType::Less)
                }
            }
            '>' => {
                if self.next_matches('=') {
                    lexeme.push(self.next().unwrap());
                    Ok(TokenType::GreaterEqual)
                } else {
                    Ok(TokenType::Greater)
                }
            }
            '=' => {
                if self.next_matches('=') {
                    lexeme.push(self.next().unwrap());
                    Ok(TokenType::EqualEqual)
                } else {
                    Ok(TokenType::Equal)
                }
            }
            '!' => {
                if self.next_matches('=') {
                    lexeme.push(self.next().unwrap());
                    Ok(TokenType::NotEqual)
                } else {
                    Ok(TokenType::Not)
                }
            }
            _ => {
                if Self::is_digit(char_rep) {
                    loop {
                        if self.next_type().is_none()
                            || !Self::is_numeric(self.peek_next().unwrap())
                        {
                            break;
                        } else {
                            lexeme.push(self.next().unwrap());
                        }
                    }

                    Ok(TokenType::Number)
                } else if Self::is_alphabetic(char_rep) {
                    loop {
                        if self.next_type().is_none()
                            || !Self::is_alphanumeric(self.peek_next().unwrap())
                        {
                            break;
                        } else {
                            lexeme.push(self.next().unwrap());
                        }
                    }

                    Ok(Self::process_identifier(&lexeme.iter().collect::<String>()))
                } else {
                    return Err("unknown character".into());
                }
            }
        }
    }

    fn add_token(&mut self, _type: TokenType, lexeme: String, line: usize, column: usize) {
        let token = Token {
            _type,
            lexeme,
            line,
            column,
        };
        self.tokens.push(token);
    }

    fn process_identifier(identifier: &str) -> TokenType {
        match identifier {
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "if" => TokenType::If,
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
        let content = "<=<>=>||&&";
        let scanner = Scanner::new(content.into()).unwrap();

        let eexpected = vec![
            (TokenType::LessEqual, "<=".to_string(), 1, 1),
            (TokenType::Less, "<".to_string(), 1, 3),
            (TokenType::GreaterEqual, ">=".to_string(), 1, 4),
            (TokenType::Greater, ">".to_string(), 1, 6),
            (TokenType::Or, "||".into(), 1, 7),
            (TokenType::And, "&&".into(), 1, 9),
        ];
        assert_expected_tokens(scanner, eexpected);
    }

    #[test]
    fn captures_identifiers_accurately() {
        let content = "class else false for if print return super true let while some_identifier someIdentifier identifier32";
        let scanner = Scanner::new(content.into()).unwrap();

        let expected = vec![
            (TokenType::Class, "class".to_string(), 1, 1),
            (TokenType::Else, "else".to_string(), 1, 7),
            (TokenType::False, "false".to_string(), 1, 12),
            (TokenType::For, "for".to_string(), 1, 18),
            (TokenType::If, "if".to_string(), 1, 22),
            (TokenType::Print, "print".to_string(), 1, 25),
            (TokenType::Return, "return".to_string(), 1, 31),
            (TokenType::Super, "super".to_string(), 1, 38),
            (TokenType::True, "true".to_string(), 1, 44),
            (TokenType::Let, "let".to_string(), 1, 49),
            (TokenType::While, "while".to_string(), 1, 53),
            (TokenType::Identifier, "some_identifier".to_string(), 1, 59),
            (TokenType::Identifier, "someIdentifier".to_string(), 1, 75),
            (TokenType::Identifier, "identifier32".to_string(), 1, 90),
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
