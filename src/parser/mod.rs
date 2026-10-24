use crate::ast::{
    BinaryOperator, Block, Expression, FunctionDeclaration, Parameter, Program, Statement, Type,
    UnaryOperator,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::source::Span;

mod codes {
    use crate::diagnostic::DiagnosticCode;
    pub const UNEXPECTED_TOKEN: DiagnosticCode = DiagnosticCode::from_u32(101);
    pub const EXPECTED_TOKEN: DiagnosticCode = DiagnosticCode::from_u32(102);
    pub const DUPLICATE_FUNCTION: DiagnosticCode = DiagnosticCode::from_u32(103);
    pub const MALFORMED_DECLARATION: DiagnosticCode = DiagnosticCode::from_u32(104);
    pub const UNTERMINATED_BLOCK: DiagnosticCode = DiagnosticCode::from_u32(105);
    pub const INVALID_GIVE_USAGE: DiagnosticCode = DiagnosticCode::from_u32(106);
    pub const PROPOSED_SYNTAX: DiagnosticCode = DiagnosticCode::from_u32(107);
}

/// Sync point tokens where error recovery can safely resume.
fn is_sync_token(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Fn
            | TokenKind::Let
            | TokenKind::Give
            | TokenKind::When
            | TokenKind::Otherwise
            | TokenKind::RightBrace
            | TokenKind::Semicolon
    )
}

/// Recursive-descent parser producing an untyped AST and parser diagnostics.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    /// Parse the token stream into a Program. Errors are collected and
    /// recovery is attempted so multiple problems can be reported at once.
    pub fn parse(mut self) -> (Program, Vec<Diagnostic>) {
        let app_metadata = if self.peek_kind() == TokenKind::App {
            self.advance();
            match self.peek_kind() {
                TokenKind::TextLiteral(value) => {
                    self.advance();
                    Some(value)
                }
                _ => {
                    self.error_at_current(
                        codes::MALFORMED_DECLARATION,
                        "expected a text literal after `app`",
                    );
                    self.skip_to_sync();
                    None
                }
            }
        } else {
            None
        };

        let mut functions = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        while !self.is_at_end() {
            if self.peek_kind() == TokenKind::Fn {
                match self.parse_function() {
                    Some(func) => {
                        if seen_names.contains(&func.name) {
                            self.errors.push(Diagnostic::error(
                                codes::DUPLICATE_FUNCTION,
                                format!("duplicate function definition `{}`", func.name),
                                func.name_span,
                            ));
                        } else {
                            seen_names.insert(func.name.clone());
                            functions.push(func);
                        }
                    }
                    None => {
                        self.skip_to_sync();
                    }
                }
            } else if self.peek_kind() == TokenKind::Flow
                || self.peek_kind() == TokenKind::Data
                || self.peek_kind() == TokenKind::Expose
            {
                let keyword = format!("{}", self.peek_kind());
                self.errors.push(Diagnostic::error(
                    codes::PROPOSED_SYNTAX,
                    format!(
                        "`{}` is proposed syntax and is not implemented in this milestone",
                        keyword
                    ),
                    self.peek_span(),
                ));
                self.advance();
                self.skip_to_sync();
            } else {
                self.error_at_current(codes::UNEXPECTED_TOKEN, "expected a function declaration");
                self.skip_to_next_declaration();
            }
        }

        let span = functions
            .first()
            .map(|f| f.span)
            .unwrap_or_else(|| Span::new(0, 0));

        (
            Program {
                app_metadata,
                functions,
                span,
            },
            self.errors,
        )
    }

    // ----- declarations -----

    fn parse_function(&mut self) -> Option<FunctionDeclaration> {
        let fn_span = self.peek_span();
        self.expect(TokenKind::Fn)?;
        let name_token = self.expect_identifier()?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => unreachable!(),
        };

        self.expect(TokenKind::LeftParen)?;
        let mut parameters = Vec::new();
        while self.peek_kind() != TokenKind::RightParen && !self.is_at_end() {
            let param = self.parse_parameter()?;
            parameters.push(param);
            if self.consume_if(TokenKind::Comma).is_none() {
                break;
            }
        }
        self.expect(TokenKind::RightParen)?;

        let (return_type, return_type_span) = if self.consume_if(TokenKind::Arrow).is_some() {
            let ty = self.parse_type()?;
            (Some(ty.clone()), Some(ty.span()))
        } else {
            (None, None)
        };

        let body = self.parse_block()?;

        let span = fn_span.merge(body.span);

        Some(FunctionDeclaration {
            name,
            name_span: name_token.span,
            parameters,
            return_type,
            return_type_span,
            body,
            span,
        })
    }

    fn parse_parameter(&mut self) -> Option<Parameter> {
        let name_token = self.expect_identifier()?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => unreachable!(),
        };
        self.expect(TokenKind::Colon)?;
        let type_annotation = self.parse_type()?;
        let type_span = type_annotation.span();
        let span = name_token.span.merge(type_span);
        Some(Parameter {
            name,
            name_span: name_token.span,
            type_annotation,
            type_span,
            span,
        })
    }

    // ----- statements -----

    fn parse_block(&mut self) -> Option<Block> {
        let brace_start = self.expect(TokenKind::LeftBrace)?.span;
        let mut statements = Vec::new();
        while self.peek_kind() != TokenKind::RightBrace && !self.is_at_end() {
            match self.parse_statement() {
                Some(stmt) => statements.push(stmt),
                None => {
                    self.skip_to_statement_end();
                    if self.peek_kind() == TokenKind::RightBrace || self.is_at_end() {
                        break;
                    }
                }
            }
        }
        self.expect(TokenKind::RightBrace)?;
        let block_end = self.prev_span();
        Some(Block {
            statements,
            span: brace_start.merge(block_end),
        })
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        let start_span = self.peek_span();
        match self.peek_kind() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Give => self.parse_give(),
            TokenKind::When => self.parse_when(),
            TokenKind::LeftBrace => {
                let block = self.parse_block()?;
                let block_span = block.span;
                Some(Statement::Block {
                    block,
                    span: start_span.merge(block_span),
                })
            }
            _ => {
                let expr = self.parse_expression()?;
                let _ = self.consume_if(TokenKind::Semicolon);
                let end = self.prev_span().end;
                Some(Statement::Expression {
                    expression: expr,
                    span: start_span.merge(Span::new(end, end)),
                })
            }
        }
    }

    fn parse_let(&mut self) -> Option<Statement> {
        let let_span = self.peek_span();
        self.expect(TokenKind::Let)?;
        let name_token = self.expect_identifier()?;
        let name = match name_token.kind {
            TokenKind::Ident(n) => n,
            _ => unreachable!(),
        };

        let (type_annotation, type_span) = if self.consume_if(TokenKind::Colon).is_some() {
            let ty = self.parse_type()?;
            (Some(ty.clone()), Some(ty.span()))
        } else {
            (None, None)
        };

        self.expect(TokenKind::Equals)?;
        let initializer = self.parse_expression()?;
        let _ = self.consume_if(TokenKind::Semicolon);

        let span = let_span.merge(initializer.span());
        Some(Statement::Let {
            name,
            name_span: name_token.span,
            type_annotation,
            type_span,
            initializer,
            span,
        })
    }

    fn parse_give(&mut self) -> Option<Statement> {
        let give_span = self.peek_span();
        self.expect(TokenKind::Give)?;
        let value = self.parse_expression()?;
        let _ = self.consume_if(TokenKind::Semicolon);
        let span = give_span.merge(value.span());
        Some(Statement::Give { value, span })
    }

    fn parse_when(&mut self) -> Option<Statement> {
        let when_span = self.peek_span();
        self.expect(TokenKind::When)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        let otherwise_branch = if self.consume_if(TokenKind::Otherwise).is_some() {
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = when_span.merge(then_branch.span);
        Some(Statement::When {
            condition,
            then_branch,
            otherwise_branch,
            span,
        })
    }

    // ----- expressions -----

    fn parse_expression(&mut self) -> Option<Expression> {
        self.parse_binary(0)
    }

    fn parse_binary(&mut self, min_precedence: u8) -> Option<Expression> {
        let mut left = self.parse_prefix()?;
        loop {
            let kind = self.peek_kind();
            let (op, precedence) = match kind {
                TokenKind::Or => (BinaryOperator::Or, 1),
                TokenKind::And => (BinaryOperator::And, 2),
                TokenKind::EqualsEquals => (BinaryOperator::Equals, 3),
                TokenKind::NotEquals => (BinaryOperator::NotEquals, 3),
                TokenKind::LessThan => (BinaryOperator::LessThan, 4),
                TokenKind::LessThanEquals => (BinaryOperator::LessThanEquals, 4),
                TokenKind::GreaterThan => (BinaryOperator::GreaterThan, 4),
                TokenKind::GreaterThanEquals => (BinaryOperator::GreaterThanEquals, 4),
                TokenKind::Plus => (BinaryOperator::Plus, 5),
                TokenKind::Minus => (BinaryOperator::Minus, 5),
                TokenKind::Star => (BinaryOperator::Star, 6),
                TokenKind::Slash => (BinaryOperator::Slash, 6),
                TokenKind::Percent => (BinaryOperator::Percent, 6),
                _ => break,
            };
            if precedence < min_precedence {
                break;
            }
            self.advance();
            let left_span = left.span();
            let right = self.parse_binary(precedence + 1)?;
            let right_span = right.span();
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
                span: left_span.merge(right_span),
            };
        }
        Some(left)
    }

    fn parse_prefix(&mut self) -> Option<Expression> {
        let kind = self.peek_kind();
        let span = self.peek_span();
        match kind {
            TokenKind::IntLiteral(value) => {
                self.advance();
                Some(Expression::IntegerLiteral { value, span })
            }
            TokenKind::FloatLiteral(value) => {
                self.advance();
                Some(Expression::FloatLiteral { value, span })
            }
            TokenKind::TextLiteral(value) => {
                self.advance();
                Some(Expression::TextLiteral { value, span })
            }
            TokenKind::True => {
                self.advance();
                Some(Expression::BooleanLiteral { value: true, span })
            }
            TokenKind::False => {
                self.advance();
                Some(Expression::BooleanLiteral { value: false, span })
            }
            TokenKind::Minus | TokenKind::Plus | TokenKind::Bang => {
                self.advance();
                let op = match kind {
                    TokenKind::Minus => UnaryOperator::Minus,
                    TokenKind::Plus => UnaryOperator::Plus,
                    TokenKind::Bang => UnaryOperator::Not,
                    _ => unreachable!(),
                };
                let operand = self.parse_prefix()?;
                let end = self.prev_span().end;
                Some(Expression::Unary {
                    operator: op,
                    operand: Box::new(operand),
                    span: span.merge(Span::new(end, end)),
                })
            }
            TokenKind::LeftParen => {
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Some(inner)
            }
            TokenKind::Ident(name) => {
                self.advance();
                if self.consume_if(TokenKind::LeftParen).is_some() {
                    let mut arguments = Vec::new();
                    while self.peek_kind() != TokenKind::RightParen && !self.is_at_end() {
                        let arg = self.parse_expression()?;
                        arguments.push(arg);
                        if self.consume_if(TokenKind::Comma).is_none() {
                            break;
                        }
                    }
                    self.expect(TokenKind::RightParen)?;
                    let end = self.prev_span().end;
                    Some(Expression::Call {
                        callee: Box::new(Expression::Identifier { name, span }),
                        arguments,
                        span: span.merge(Span::new(end, end)),
                    })
                } else {
                    Some(Expression::Identifier { name, span })
                }
            }
            TokenKind::Flow | TokenKind::Data | TokenKind::Expose => {
                self.error_at_current(
                    codes::PROPOSED_SYNTAX,
                    format!(
                        "`{}` is proposed syntax and is not implemented in this milestone",
                        kind
                    ),
                );
                self.advance();
                None
            }
            _ => {
                self.error_at_current(
                    codes::UNEXPECTED_TOKEN,
                    format!("expected an expression, found `{}`", kind),
                );
                None
            }
        }
    }

    fn parse_type(&mut self) -> Option<Type> {
        let kind = self.peek_kind();
        let ty = match kind {
            TokenKind::IntKw => Type::Int,
            TokenKind::FloatKw => Type::Float,
            TokenKind::BoolKw => Type::Bool,
            TokenKind::TextKw => Type::Text,
            TokenKind::UnitKw => Type::Unit,
            _ => {
                self.error_at_current(codes::EXPECTED_TOKEN, "expected a type");
                return None;
            }
        };
        self.advance();
        Some(ty)
    }

    // ----- helpers -----

    fn peek_kind(&self) -> TokenKind {
        self.peek().kind.clone()
    }

    fn peek_span(&self) -> Span {
        self.peek().span
    }

    fn prev_span(&self) -> Span {
        let idx = if self.pos == 0 { 0 } else { self.pos - 1 };
        self.tokens[idx].span
    }

    fn at(&self, kind: &TokenKind) -> bool {
        &self.peek().kind == kind
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
            || (self.peek_kind() == TokenKind::Ident(String::new()) && self.peek_span().is_empty())
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(&kind) {
            Some(self.advance())
        } else {
            self.error_at_current(
                codes::EXPECTED_TOKEN,
                format!("expected `{}`, found `{}`", kind, self.peek_kind()),
            );
            None
        }
    }

    fn expect_identifier(&mut self) -> Option<Token> {
        match self.peek_kind() {
            TokenKind::Ident(_) => Some(self.advance()),
            _ => {
                self.error_at_current(codes::EXPECTED_TOKEN, "expected an identifier");
                None
            }
        }
    }

    fn consume_if(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(&kind) {
            Some(self.advance())
        } else {
            None
        }
    }

    fn error_at_current(&mut self, code: DiagnosticCode, message: impl Into<String>) {
        let token = self.peek();
        self.errors
            .push(Diagnostic::error(code, message.into(), token.span));
    }

    /// Skip tokens until a sync point, consuming the sync token itself if it is
    /// a statement terminator.
    fn skip_to_sync(&mut self) {
        while !self.is_at_end() {
            let kind = self.peek_kind();
            if is_sync_token(&kind) {
                if kind == TokenKind::Semicolon || kind == TokenKind::RightBrace {
                    self.advance();
                }
                break;
            }
            self.advance();
        }
    }

    /// Skip past tokens that are not top-level declarations, stopping before
    /// the next `fn` or the end. Guarantees forward progress so error recovery
    /// cannot loop on a statement-level sync token such as `let`.
    fn skip_to_next_declaration(&mut self) {
        while !self.is_at_end() && self.peek_kind() != TokenKind::Fn {
            self.advance();
        }
    }

    /// Skip to the end of the current statement (semicolon or closing brace).
    fn skip_to_statement_end(&mut self) {
        while !self.is_at_end() {
            let kind = self.peek_kind();
            if kind == TokenKind::Semicolon
                || kind == TokenKind::RightBrace
                || kind == TokenKind::RightParen
            {
                break;
            }
            self.advance();
        }
    }
}
