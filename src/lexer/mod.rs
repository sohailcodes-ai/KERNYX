use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::source::{Location, Source, Span};

pub mod token;
use crate::lexer::token::{Token, TokenKind};

/// Error codes used by the lexer.
mod codes {
    use crate::diagnostic::DiagnosticCode;
    pub const INVALID_CHARACTER: DiagnosticCode = DiagnosticCode::from_u32(1001);
    pub const UNTERMINATED_STRING: DiagnosticCode = DiagnosticCode::from_u32(1002);
    pub const INVALID_ESCAPE: DiagnosticCode = DiagnosticCode::from_u32(1003);
    pub const NUMBER_TOO_LARGE: DiagnosticCode = DiagnosticCode::from_u32(1004);
    pub const INVALID_FLOAT: DiagnosticCode = DiagnosticCode::from_u32(1005);
}

/// The lexer turns source text into a stream of tokens and lexing diagnostics.
pub struct Lexer<'a> {
    source: &'a Source,
    bytes: &'a [u8],
    position: usize,
    line: usize,
    column: usize,
    errors: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a Source) -> Lexer<'a> {
        Lexer {
            source,
            bytes: source.text.as_bytes(),
            position: 0,
            line: 1,
            column: 1,
            errors: Vec::new(),
        }
    }

    /// Lex the whole source, returning tokens and any diagnostics.
    pub fn lex(mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        let mut tokens = Vec::new();
        while self.position < self.bytes.len() {
            match self.next_token() {
                Ok(Some(token)) => tokens.push(token),
                Ok(None) => {}
                Err(error) => self.errors.push(error),
            }
        }
        tokens.push(Token::new(
            TokenKind::Ident(String::new()),
            Span::new(self.position, self.position),
            self.current_location(),
        ));
        (tokens, self.errors)
    }

    fn current_location(&self) -> Location {
        Location {
            line: self.line,
            column: self.column,
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>, Diagnostic> {
        self.skip_whitespace_and_comments();
        if self.position >= self.bytes.len() {
            return Ok(None);
        }

        let start = self.position;
        let location = self.current_location();
        let b = self.bytes[start];

        match b {
            b'(' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::LeftParen,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b')' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::RightParen,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b'{' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::LeftBrace,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b'}' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::RightBrace,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b',' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::Comma,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b';' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::Semicolon,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b':' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::Colon,
                    Span::new(start, start + 1),
                    location,
                )))
            }

            b'+' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::Plus,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b'-' => {
                if self.peek() == b'>' {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::Arrow,
                        Span::new(start, start + 2),
                        location,
                    )))
                } else {
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::Minus,
                        Span::new(start, start + 1),
                        location,
                    )))
                }
            }
            b'*' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::Star,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b'/' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::Slash,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b'%' => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::Percent,
                    Span::new(start, start + 1),
                    location,
                )))
            }
            b'!' => {
                if self.peek() == b'=' {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::NotEquals,
                        Span::new(start, start + 2),
                        location,
                    )))
                } else {
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::Bang,
                        Span::new(start, start + 1),
                        location,
                    )))
                }
            }
            b'=' => {
                if self.peek() == b'=' {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::EqualsEquals,
                        Span::new(start, start + 2),
                        location,
                    )))
                } else {
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::Equals,
                        Span::new(start, start + 1),
                        location,
                    )))
                }
            }
            b'<' => {
                if self.peek() == b'=' {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::LessThanEquals,
                        Span::new(start, start + 2),
                        location,
                    )))
                } else {
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::LessThan,
                        Span::new(start, start + 1),
                        location,
                    )))
                }
            }
            b'>' => {
                if self.peek() == b'=' {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::GreaterThanEquals,
                        Span::new(start, start + 2),
                        location,
                    )))
                } else {
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::GreaterThan,
                        Span::new(start, start + 1),
                        location,
                    )))
                }
            }
            b'&' => {
                if self.peek() == b'&' {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::And,
                        Span::new(start, start + 2),
                        location,
                    )))
                } else {
                    Err(self.invalid_character_error('&', Span::new(start, start + 1)))
                }
            }
            b'|' => {
                if self.peek() == b'|' {
                    self.advance();
                    self.advance();
                    Ok(Some(Token::new(
                        TokenKind::Or,
                        Span::new(start, start + 2),
                        location,
                    )))
                } else {
                    Err(self.invalid_character_error('|', Span::new(start, start + 1)))
                }
            }

            b'"' => self.read_text(start, location),

            b'0'..=b'9' => self.read_number(start, location),

            b if is_ident_start(b) => self.read_ident(start, location),

            _ => {
                self.advance();
                Err(self.invalid_character_error(b as char, Span::new(start, start + 1)))
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while self.position < self.bytes.len() {
                let b = self.bytes[self.position];
                match b {
                    b' ' | b'\t' | b'\r' => {
                        self.advance();
                    }
                    b'\n' => {
                        self.advance_newline();
                    }
                    _ => break,
                }
            }
            if self.position + 1 < self.bytes.len()
                && self.bytes[self.position] == b'/'
                && self.bytes[self.position + 1] == b'/'
            {
                self.skip_line_comment();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();
        while self.position < self.bytes.len() && self.bytes[self.position] != b'\n' {
            self.advance();
        }
    }

    fn advance(&mut self) {
        if self.position < self.bytes.len() {
            if self.bytes[self.position] == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn advance_newline(&mut self) {
        self.position += 1;
        self.line += 1;
        self.column = 1;
    }

    fn peek(&self) -> u8 {
        if self.position + 1 < self.bytes.len() {
            self.bytes[self.position + 1]
        } else {
            0
        }
    }

    fn read_ident(
        &mut self,
        start: usize,
        location: Location,
    ) -> Result<Option<Token>, Diagnostic> {
        while self.position < self.bytes.len() && is_ident_cont(self.bytes[self.position]) {
            self.advance();
        }
        let end = self.position;
        let text = &self.source.text[start..end];
        let kind = match TokenKind::is_keyword(text) {
            Some(kind) => kind,
            None => TokenKind::Ident(text.to_string()),
        };
        Ok(Some(Token::new(kind, Span::new(start, end), location)))
    }

    fn read_number(
        &mut self,
        start: usize,
        location: Location,
    ) -> Result<Option<Token>, Diagnostic> {
        let mut is_float = false;
        while self.position < self.bytes.len() {
            let b = self.bytes[self.position];
            if b == b'.' {
                if self.position + 1 < self.bytes.len()
                    && self.bytes[self.position + 1].is_ascii_digit()
                {
                    is_float = true;
                    self.advance();
                } else {
                    break;
                }
            } else if b.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.position;
        let text = &self.source.text[start..end];
        if is_float {
            match text.parse::<f64>() {
                Ok(value) => Ok(Some(Token::new(
                    TokenKind::FloatLiteral(value),
                    Span::new(start, end),
                    location,
                ))),
                Err(_) => Err(self.diagnostic(
                    codes::INVALID_FLOAT,
                    "invalid floating-point literal".to_string(),
                    Span::new(start, end),
                )),
            }
        } else {
            match text.parse::<i64>() {
                Ok(value) => Ok(Some(Token::new(
                    TokenKind::IntLiteral(value),
                    Span::new(start, end),
                    location,
                ))),
                Err(_) => Err(self.diagnostic(
                    codes::NUMBER_TOO_LARGE,
                    "integer literal too large".to_string(),
                    Span::new(start, end),
                )),
            }
        }
    }

    fn read_text(&mut self, start: usize, location: Location) -> Result<Option<Token>, Diagnostic> {
        self.advance();
        let mut value = String::new();
        while self.position < self.bytes.len() {
            let b = self.bytes[self.position];
            if b == b'"' {
                self.advance();
                return Ok(Some(Token::new(
                    TokenKind::TextLiteral(value),
                    Span::new(start, self.position),
                    location,
                )));
            }
            if b == b'\\' {
                let escape_start = self.position;
                self.advance();
                if self.position >= self.bytes.len() {
                    return Err(self.diagnostic(
                        codes::UNTERMINATED_STRING,
                        "unterminated string literal".to_string(),
                        Span::new(start, self.position),
                    ));
                }
                let escaped = self.bytes[self.position];
                match escaped {
                    b'n' => value.push('\n'),
                    b't' => value.push('\t'),
                    b'"' => value.push('"'),
                    b'\\' => value.push('\\'),
                    other => {
                        return Err(self.diagnostic(
                            codes::INVALID_ESCAPE,
                            format!("invalid escape sequence `\\{}`", other as char),
                            Span::new(escape_start, self.position + 1),
                        ));
                    }
                }
                self.advance();
                continue;
            }
            if b == b'\n' {
                return Err(self.diagnostic(
                    codes::UNTERMINATED_STRING,
                    "unterminated string literal".to_string(),
                    Span::new(start, self.position),
                ));
            }
            value.push(b as char);
            self.advance();
        }
        Err(self.diagnostic(
            codes::UNTERMINATED_STRING,
            "unterminated string literal".to_string(),
            Span::new(start, self.position),
        ))
    }

    fn invalid_character_error(&self, c: char, span: Span) -> Diagnostic {
        self.diagnostic(
            codes::INVALID_CHARACTER,
            format!("invalid character `{}`", c),
            span,
        )
    }

    fn diagnostic(&self, code: DiagnosticCode, message: String, span: Span) -> Diagnostic {
        Diagnostic::error(code, message, span)
    }
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_cont(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::TokenKind;
    use crate::source::Source;

    fn lex(text: &str) -> (Vec<Token>, Vec<Diagnostic>) {
        let source = Source::new("test.knx", text.to_string());
        Lexer::new(&source).lex()
    }

    #[test]
    fn tokens_for_hello() {
        let src = r#"app "hello"

fn greet(name: Text) -> Text {
    give "yo, " + name
}

fn main() {
    let user = greet("world")

    when user != "" {
        print(user)
    } otherwise {
        print("nah, nothing here")
    }
}
"#;
        let (tokens, errors) = lex(src);
        assert!(errors.is_empty(), "unexpected lex errors: {:?}", errors);
        let kinds: Vec<String> = tokens.iter().map(|t| format!("{}", t.kind)).collect();
        assert!(kinds.iter().any(|k| k == "app"));
        assert!(kinds.iter().any(|k| k == "fn"));
        assert!(kinds.iter().any(|k| k == "Text"));
        assert!(kinds.iter().any(|k| k == "\"yo, \""));
        assert!(kinds.iter().any(|k| k == "+"));
        assert!(kinds.iter().any(|k| k == "!="));
    }

    #[test]
    fn unterminated_string_is_error() {
        let (_, errors) = lex("\"unterminated");
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.code == codes::UNTERMINATED_STRING));
    }

    #[test]
    fn invalid_character_is_error() {
        let (_, errors) = lex("let x = @");
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.code == codes::INVALID_CHARACTER));
    }

    #[test]
    fn invalid_escape_is_error() {
        let (_, errors) = lex(r#""\q""#);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.code == codes::INVALID_ESCAPE));
    }

    #[test]
    fn escapes_are_decoded() {
        let (tokens, errors) = lex(r#" "a\nb\tc""#);
        assert!(errors.is_empty());
        match &tokens[0].kind {
            TokenKind::TextLiteral(value) => assert_eq!(value, "a\nb\tc"),
            other => panic!("expected text literal, got {:?}", other),
        }
    }

    #[test]
    fn number_parsing() {
        let pi = std::f64::consts::PI.to_string();
        let (tokens, errors) = lex(&format!("42 {} -7", pi));
        assert!(errors.is_empty());
        match &tokens[0].kind {
            TokenKind::IntLiteral(n) => assert_eq!(*n, 42),
            other => panic!("expected int, got {:?}", other),
        }
        match &tokens[1].kind {
            TokenKind::FloatLiteral(n) => assert!((*n - std::f64::consts::PI).abs() < f64::EPSILON),
            other => panic!("expected float, got {:?}", other),
        }
    }

    #[test]
    fn source_spans_tracked() {
        let (tokens, _errors) = lex("ab cd");
        assert_eq!(tokens[0].span, Span::new(0, 2));
        assert_eq!(tokens[0].location, Location { line: 1, column: 1 });
        assert_eq!(tokens[1].span, Span::new(3, 5));
        assert_eq!(tokens[1].location, Location { line: 1, column: 4 });
    }
}
