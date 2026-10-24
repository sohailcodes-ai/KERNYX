use crate::source::Span;

/// A whole KERNYX source program.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub app_metadata: Option<String>,
    pub functions: Vec<FunctionDeclaration>,
    pub span: Span,
}

/// A function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclaration {
    pub name: String,
    pub name_span: Span,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub return_type_span: Option<Span>,
    pub body: Block,
    pub span: Span,
}

/// A typed function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub name: String,
    pub name_span: Span,
    pub type_annotation: Type,
    pub type_span: Span,
    pub span: Span,
}

/// A block of statements.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        name_span: Span,
        type_annotation: Option<Type>,
        type_span: Option<Span>,
        initializer: Expression,
        span: Span,
    },
    Expression {
        expression: Expression,
        span: Span,
    },
    Block {
        block: Block,
        span: Span,
    },
    Give {
        value: Expression,
        span: Span,
    },
    When {
        condition: Expression,
        then_branch: Block,
        otherwise_branch: Option<Block>,
        span: Span,
    },
}

/// Expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    IntegerLiteral {
        value: i64,
        span: Span,
    },
    FloatLiteral {
        value: f64,
        span: Span,
    },
    TextLiteral {
        value: String,
        span: Span,
    },
    BooleanLiteral {
        value: bool,
        span: Span,
    },
    Identifier {
        name: String,
        span: Span,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
        span: Span,
    },
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
        span: Span,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Minus,
    Plus,
    Not,
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::IntegerLiteral { span, .. }
            | Expression::FloatLiteral { span, .. }
            | Expression::TextLiteral { span, .. }
            | Expression::BooleanLiteral { span, .. }
            | Expression::Identifier { span, .. }
            | Expression::Unary { span, .. }
            | Expression::Binary { span, .. }
            | Expression::Call { span, .. } => *span,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Text,
    Unit,
}

impl Type {
    pub fn name(&self) -> &'static str {
        match self {
            Type::Int => "Int",
            Type::Float => "Float",
            Type::Bool => "Bool",
            Type::Text => "Text",
            Type::Unit => "Unit",
        }
    }

    pub fn span(&self) -> Span {
        Span::new(0, 0)
    }

    /// Whether a value of this type can be returned from a function whose
    /// declared return type is `other`. For this milestone types are nominal
    /// and exactly equal (no implicit Int -> Float conversion).
    pub fn assignable_to(&self, other: &Type) -> bool {
        self == other
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl BinaryOperator {
    pub fn precedence(self) -> u8 {
        match self {
            BinaryOperator::Or => 1,
            BinaryOperator::And => 2,
            BinaryOperator::Equals | BinaryOperator::NotEquals => 3,
            BinaryOperator::LessThan
            | BinaryOperator::LessThanEquals
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanEquals => 4,
            BinaryOperator::Plus | BinaryOperator::Minus => 5,
            BinaryOperator::Star | BinaryOperator::Slash | BinaryOperator::Percent => 6,
        }
    }

    pub fn is_associative_left(self) -> bool {
        true
    }
}
