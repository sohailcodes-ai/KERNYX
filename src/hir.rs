use crate::ast::Type;
use crate::source::Span;

/// A typed, validation-ready representation of a KERNYX program.
///
/// The HIR is the stable boundary between the frontend (lexer, parser, type
/// checker) and backends/interpreters. Future native and WASM backends consume
/// this representation rather than re-parsing source.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedProgram {
    pub app_metadata: Option<String>,
    pub functions: Vec<TypedFunction>,
    pub span: Span,
}

/// A type-checked function.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedFunction {
    pub name: String,
    pub name_span: Span,
    pub parameters: Vec<TypedParameter>,
    pub return_type: Type,
    pub body: TypedBlock,
    pub span: Span,
}

/// A typed function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParameter {
    pub name: String,
    pub type_: Type,
    pub span: Span,
}

/// A typed block of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedBlock {
    pub statements: Vec<TypedStatement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedStatement {
    Let {
        name: String,
        type_: Type,
        initializer: TypedExpression,
        span: Span,
    },
    Expression {
        expression: TypedExpression,
        span: Span,
    },
    Block {
        block: TypedBlock,
        span: Span,
    },
    Give {
        value: TypedExpression,
        span: Span,
    },
    When {
        condition: TypedExpression,
        then_branch: TypedBlock,
        otherwise_branch: Option<TypedBlock>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpression {
    IntegerLiteral {
        value: i64,
        type_: Type,
        span: Span,
    },
    FloatLiteral {
        value: f64,
        type_: Type,
        span: Span,
    },
    TextLiteral {
        value: String,
        type_: Type,
        span: Span,
    },
    BooleanLiteral {
        value: bool,
        type_: Type,
        span: Span,
    },
    Var {
        name: String,
        type_: Type,
        span: Span,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<TypedExpression>,
        type_: Type,
        span: Span,
    },
    Binary {
        left: Box<TypedExpression>,
        operator: BinaryOperator,
        right: Box<TypedExpression>,
        type_: Type,
        span: Span,
    },
    Call {
        callee: String,
        arguments: Vec<TypedExpression>,
        return_type: Type,
        span: Span,
    },
}

impl TypedExpression {
    pub fn type_(&self) -> Type {
        match self {
            TypedExpression::IntegerLiteral { type_, .. } => type_.clone(),
            TypedExpression::FloatLiteral { type_, .. } => type_.clone(),
            TypedExpression::TextLiteral { type_, .. } => type_.clone(),
            TypedExpression::BooleanLiteral { type_, .. } => type_.clone(),
            TypedExpression::Var { type_, .. } => type_.clone(),
            TypedExpression::Unary { type_, .. } => type_.clone(),
            TypedExpression::Binary { type_, .. } => type_.clone(),
            TypedExpression::Call { return_type, .. } => return_type.clone(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            TypedExpression::IntegerLiteral { span, .. }
            | TypedExpression::FloatLiteral { span, .. }
            | TypedExpression::TextLiteral { span, .. }
            | TypedExpression::BooleanLiteral { span, .. }
            | TypedExpression::Var { span, .. }
            | TypedExpression::Unary { span, .. }
            | TypedExpression::Binary { span, .. }
            | TypedExpression::Call { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Minus,
    Plus,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equals,
    NotEquals,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    And,
    Or,
}
