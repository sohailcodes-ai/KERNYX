use crate::source::{Location, Span};

/// A single token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub location: Location,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, location: Location) -> Token {
        Token {
            kind,
            span,
            location,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    App,
    Fn,
    Let,
    Give,
    When,
    Otherwise,
    Return,
    TextKw,
    IntKw,
    FloatKw,
    BoolKw,
    UnitKw,
    True,
    False,
    // Proposed-but-unimplemented keywords (lexed but rejected by the parser).
    Flow,
    Data,
    Expose,

    // Identifiers and literals
    Ident(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    TextLiteral(String),

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Colon,
    Arrow, // ->

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Equals,
    EqualsEquals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    And,
    Or,
}

impl TokenKind {
    pub fn is_keyword(keyword: &str) -> Option<TokenKind> {
        match keyword {
            "app" => Some(TokenKind::App),
            "fn" => Some(TokenKind::Fn),
            "let" => Some(TokenKind::Let),
            "give" => Some(TokenKind::Give),
            "when" => Some(TokenKind::When),
            "otherwise" => Some(TokenKind::Otherwise),
            "return" => Some(TokenKind::Return),
            "Text" => Some(TokenKind::TextKw),
            "Int" => Some(TokenKind::IntKw),
            "Float" => Some(TokenKind::FloatKw),
            "Bool" => Some(TokenKind::BoolKw),
            "Unit" => Some(TokenKind::UnitKw),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "flow" => Some(TokenKind::Flow),
            "data" => Some(TokenKind::Data),
            "expose" => Some(TokenKind::Expose),
            _ => None,
        }
    }

    pub fn precedence(self) -> Option<u8> {
        match self {
            TokenKind::Or => Some(1),
            TokenKind::And => Some(2),
            TokenKind::EqualsEquals | TokenKind::NotEquals => Some(3),
            TokenKind::LessThan
            | TokenKind::LessThanEquals
            | TokenKind::GreaterThan
            | TokenKind::GreaterThanEquals => Some(4),
            TokenKind::Plus | TokenKind::Minus => Some(5),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(6),
            _ => None,
        }
    }

    pub fn is_unary_prefix(&self) -> bool {
        matches!(self, TokenKind::Minus | TokenKind::Plus | TokenKind::Bang)
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::App => write!(f, "app"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Give => write!(f, "give"),
            TokenKind::When => write!(f, "when"),
            TokenKind::Otherwise => write!(f, "otherwise"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::TextKw => write!(f, "Text"),
            TokenKind::IntKw => write!(f, "Int"),
            TokenKind::FloatKw => write!(f, "Float"),
            TokenKind::BoolKw => write!(f, "Bool"),
            TokenKind::UnitKw => write!(f, "Unit"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Flow => write!(f, "flow"),
            TokenKind::Data => write!(f, "data"),
            TokenKind::Expose => write!(f, "expose"),
            TokenKind::Ident(name) => write!(f, "{}", name),
            TokenKind::IntLiteral(n) => write!(f, "{}", n),
            TokenKind::FloatLiteral(n) => write!(f, "{}", n),
            TokenKind::TextLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Bang => write!(f, "!"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::EqualsEquals => write!(f, "=="),
            TokenKind::NotEquals => write!(f, "!="),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::LessThanEquals => write!(f, "<="),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::GreaterThanEquals => write!(f, ">="),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
        }
    }
}

use std::fmt;

/// A lexing error with a source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    InvalidCharacter { char: char, span: Span },
    UnterminatedString { span: Span },
    UnterminatedComment { span: Span },
    InvalidEscape { char: char, span: Span },
    NumberTooLarge { span: Span },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::InvalidCharacter { char, .. } => write!(f, "invalid character `{}`", char),
            LexError::UnterminatedString { .. } => write!(f, "unterminated string literal"),
            LexError::UnterminatedComment { .. } => write!(f, "unterminated comment"),
            LexError::InvalidEscape { char, .. } => {
                write!(f, "invalid escape sequence `\\{}`", char)
            }
            LexError::NumberTooLarge { .. } => write!(f, "integer literal too large"),
        }
    }
}
