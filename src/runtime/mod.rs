use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::hir::{
    BinaryOperator, TypedBlock, TypedExpression, TypedFunction, TypedProgram, TypedStatement,
    UnaryOperator,
};
use crate::source::Span;
use std::collections::HashMap;

mod codes {
    use crate::diagnostic::DiagnosticCode;
    pub const DIVISION_BY_ZERO: DiagnosticCode = DiagnosticCode::from_u32(301);
    pub const INTEGER_OVERFLOW: DiagnosticCode = DiagnosticCode::from_u32(302);
    pub const RECURSION_LIMIT: DiagnosticCode = DiagnosticCode::from_u32(303);
    pub const RUNTIME_ERROR: DiagnosticCode = DiagnosticCode::from_u32(304);
    pub const INVALID_OPERAND_TYPES: DiagnosticCode = DiagnosticCode::from_u32(305);
    pub const UNDEFINED_VARIABLE: DiagnosticCode = DiagnosticCode::from_u32(306);
}

/// A KERNYX runtime value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Text(String),
    Bool(bool),
    Unit,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Text(_) => "Text",
            Value::Bool(_) => "Bool",
            Value::Unit => "Unit",
        }
    }
}

/// Documented recursion limit to prevent unbounded recursion from exhausting
/// the stack.
const MAX_CALL_DEPTH: u32 = 1024;

/// The interpreter executes a validated, typed KERNYX program.
pub struct Interpreter {
    program: TypedProgram,
    functions: HashMap<String, TypedFunction>,
    globals: HashMap<String, Value>,
    call_depth: u32,
}

impl Interpreter {
    pub fn new(program: TypedProgram) -> Interpreter {
        let mut functions = HashMap::new();
        for func in &program.functions {
            functions.insert(func.name.clone(), func.clone());
        }
        Interpreter {
            functions,
            program,
            globals: HashMap::new(),
            call_depth: 0,
        }
    }

    /// Execute the program, invoking `main`. Program output is written to
    /// stdout via the `print` builtin.
    pub fn run(&mut self) -> Result<(), Diagnostic> {
        let main = self.functions.get("main").cloned().ok_or_else(|| {
            Diagnostic::error(
                codes::RUNTIME_ERROR,
                "no `main` function found at runtime",
                self.program.span,
            )
        })?;

        self.call_depth += 1;
        let result = self.call_function(&main, Vec::new(), main.span);
        self.call_depth -= 1;

        result?;
        Ok(())
    }

    fn call_function(
        &mut self,
        func: &TypedFunction,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, Diagnostic> {
        if self.call_depth > MAX_CALL_DEPTH {
            return Err(Diagnostic::error(
                codes::RECURSION_LIMIT,
                format!(
                    "maximum call depth ({}) exceeded; possible infinite recursion",
                    MAX_CALL_DEPTH
                ),
                span,
            ));
        }

        if func.parameters.len() != arguments.len() {
            return Err(Diagnostic::error(
                codes::RUNTIME_ERROR,
                format!(
                    "function `{}` expects {} arguments, got {}",
                    func.name,
                    func.parameters.len(),
                    arguments.len()
                ),
                span,
            ));
        }

        let mut locals = HashMap::new();
        for (param, arg) in func.parameters.iter().zip(arguments) {
            locals.insert(param.name.clone(), arg);
        }

        let previous = std::mem::replace(&mut self.globals, locals);
        let call_depth = self.call_depth;
        self.call_depth = call_depth + 1;
        let result = self.execute_block(&func.body);
        self.call_depth = call_depth;
        self.globals = previous;

        result
    }

    fn execute_block(&mut self, block: &TypedBlock) -> Result<Value, Diagnostic> {
        let mut last = Value::Unit;
        for stmt in &block.statements {
            last = self.execute_statement(stmt)?;
        }
        Ok(last)
    }

    fn execute_statement(&mut self, stmt: &TypedStatement) -> Result<Value, Diagnostic> {
        match stmt {
            TypedStatement::Let {
                name, initializer, ..
            } => {
                let value = self.evaluate_expression(initializer)?;
                self.globals.insert(name.clone(), value);
                Ok(Value::Unit)
            }
            TypedStatement::Expression { expression, .. } => self.evaluate_expression(expression),
            TypedStatement::Block { block, .. } => self.execute_block(block),
            TypedStatement::Give { value, span: _ } => {
                let value = self.evaluate_expression(value)?;
                Ok(value)
            }
            TypedStatement::When {
                condition,
                then_branch,
                otherwise_branch,
                span,
            } => {
                let cond = self.evaluate_expression(condition)?;
                match cond {
                    Value::Bool(true) => self.execute_block(then_branch),
                    Value::Bool(false) => match otherwise_branch {
                        Some(block) => self.execute_block(block),
                        None => Ok(Value::Unit),
                    },
                    other => Err(Diagnostic::error(
                        codes::RUNTIME_ERROR,
                        format!(
                            "when condition evaluated to non-boolean value `{:?}`",
                            other
                        ),
                        *span,
                    )),
                }
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &TypedExpression) -> Result<Value, Diagnostic> {
        match expr {
            TypedExpression::IntegerLiteral { value, .. } => Ok(Value::Int(*value)),
            TypedExpression::FloatLiteral { value, .. } => Ok(Value::Float(*value)),
            TypedExpression::TextLiteral { value, .. } => Ok(Value::Text(value.clone())),
            TypedExpression::BooleanLiteral { value, .. } => Ok(Value::Bool(*value)),
            TypedExpression::Var { name, .. } => self.globals.get(name).cloned().ok_or_else(|| {
                Diagnostic::error(
                    DiagnosticCode::from_u32(305),
                    format!("undefined variable `{}`", name),
                    expr.span(),
                )
            }),
            TypedExpression::Unary {
                operator,
                operand,
                span,
                ..
            } => {
                let inner = self.evaluate_expression(operand)?;
                match operator {
                    UnaryOperator::Minus => match inner {
                        Value::Int(v) => Ok(Value::Int(checked_neg(v, *span)?)),
                        Value::Float(v) => Ok(Value::Float(-v)),
                        other => Err(runtime_error(
                            codes::RUNTIME_ERROR,
                            format!("cannot negate `{}`", other.type_name()),
                            *span,
                        )),
                    },
                    UnaryOperator::Plus => Ok(inner),
                    UnaryOperator::Not => match inner {
                        Value::Bool(v) => Ok(Value::Bool(!v)),
                        other => Err(runtime_error(
                            codes::RUNTIME_ERROR,
                            format!("cannot apply `!` to `{}`", other.type_name()),
                            *span,
                        )),
                    },
                }
            }
            TypedExpression::Binary {
                left,
                operator,
                right,
                span,
                ..
            } => {
                let l = self.evaluate_expression(left)?;
                let r = self.evaluate_expression(right)?;
                self.apply_binary(operator, l, r, *span)
            }
            TypedExpression::Call {
                callee,
                arguments,
                return_type: _,
                span,
            } => {
                if callee == "print" {
                    if arguments.len() != 1 {
                        return Err(Diagnostic::error(
                            codes::RUNTIME_ERROR,
                            "print takes exactly 1 argument",
                            *span,
                        ));
                    }
                    let value = self.evaluate_expression(&arguments[0])?;
                    println!("{}", value_display(&value));
                    return Ok(Value::Unit);
                }

                let func = self.functions.get(callee).cloned().ok_or_else(|| {
                    Diagnostic::error(
                        codes::RUNTIME_ERROR,
                        format!("undefined function `{}`", callee),
                        *span,
                    )
                })?;

                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.evaluate_expression(arg)?);
                }

                self.call_function(&func, args, *span)
            }
        }
    }

    fn apply_binary(
        &self,
        operator: &BinaryOperator,
        left: Value,
        right: Value,
        span: Span,
    ) -> Result<Value, Diagnostic> {
        use BinaryOperator::*;
        match (operator, left, right) {
            (Plus, Value::Int(a), Value::Int(b)) => Ok(Value::Int(checked_add(a, b, span)?)),
            (Minus, Value::Int(a), Value::Int(b)) => Ok(Value::Int(checked_sub(a, b, span)?)),
            (Star, Value::Int(a), Value::Int(b)) => Ok(Value::Int(checked_mul(a, b, span)?)),
            (Slash, Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    return Err(Diagnostic::error(
                        codes::DIVISION_BY_ZERO,
                        "integer division by zero",
                        span,
                    ));
                }
                Ok(Value::Int(a.checked_div(b).ok_or_else(|| {
                    Diagnostic::error(codes::INTEGER_OVERFLOW, "integer overflow", span)
                })?))
            }
            (Percent, Value::Int(a), Value::Int(b)) => {
                if b == 0 {
                    return Err(Diagnostic::error(
                        codes::DIVISION_BY_ZERO,
                        "integer modulo by zero",
                        span,
                    ));
                }
                Ok(Value::Int(a % b))
            }
            (Plus, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Minus, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Star, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Slash, Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    return Err(Diagnostic::error(
                        codes::DIVISION_BY_ZERO,
                        "floating-point division by zero",
                        span,
                    ));
                }
                Ok(Value::Float(a / b))
            }
            (Plus, Value::Text(a), Value::Text(b)) => Ok(Value::Text(format!("{}{}", a, b))),
            (Equals, a, b) => Ok(Value::Bool(value_eq(&a, &b))),
            (NotEquals, a, b) => Ok(Value::Bool(!value_eq(&a, &b))),
            (LessThan, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (LessThanEquals, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (GreaterThan, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (GreaterThanEquals, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (LessThan, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (LessThanEquals, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (GreaterThan, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (GreaterThanEquals, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (LessThan, Value::Text(a), Value::Text(b)) => Ok(Value::Bool(a < b)),
            (LessThanEquals, Value::Text(a), Value::Text(b)) => Ok(Value::Bool(a <= b)),
            (GreaterThan, Value::Text(a), Value::Text(b)) => Ok(Value::Bool(a > b)),
            (GreaterThanEquals, Value::Text(a), Value::Text(b)) => Ok(Value::Bool(a >= b)),
            (And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            (Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            (op, l, r) => Err(runtime_error(
                codes::INVALID_OPERAND_TYPES,
                format!(
                    "cannot apply `{}` to `{}` and `{}`",
                    operator_symbol(op),
                    l.type_name(),
                    r.type_name()
                ),
                span,
            )),
        }
    }
}

fn checked_add(a: i64, b: i64, span: Span) -> Result<i64, Diagnostic> {
    a.checked_add(b).ok_or_else(|| {
        Diagnostic::error(
            codes::INTEGER_OVERFLOW,
            format!("integer overflow: {} + {}", a, b),
            span,
        )
    })
}

fn checked_sub(a: i64, b: i64, span: Span) -> Result<i64, Diagnostic> {
    a.checked_sub(b).ok_or_else(|| {
        Diagnostic::error(
            codes::INTEGER_OVERFLOW,
            format!("integer overflow: {} - {}", a, b),
            span,
        )
    })
}

fn checked_mul(a: i64, b: i64, span: Span) -> Result<i64, Diagnostic> {
    a.checked_mul(b).ok_or_else(|| {
        Diagnostic::error(
            codes::INTEGER_OVERFLOW,
            format!("integer overflow: {} * {}", a, b),
            span,
        )
    })
}

fn checked_neg(a: i64, span: Span) -> Result<i64, Diagnostic> {
    a.checked_neg().ok_or_else(|| {
        Diagnostic::error(codes::INTEGER_OVERFLOW, "integer overflow: negation", span)
    })
}

fn value_eq(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Text(a), Value::Text(b)) => a == b,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Unit, Value::Unit) => true,
        _ => false,
    }
}

fn value_display(value: &Value) -> String {
    match value {
        Value::Int(v) => v.to_string(),
        Value::Float(v) => {
            if v.fract() == 0.0 {
                format!("{:.1}", v)
            } else {
                v.to_string()
            }
        }
        Value::Text(v) => v.clone(),
        Value::Bool(v) => v.to_string(),
        Value::Unit => "()".to_string(),
    }
}

fn runtime_error(code: DiagnosticCode, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::error(code, message.into(), span)
}

fn operator_symbol(operator: &BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Plus => "+",
        BinaryOperator::Minus => "-",
        BinaryOperator::Star => "*",
        BinaryOperator::Slash => "/",
        BinaryOperator::Percent => "%",
        BinaryOperator::Equals => "==",
        BinaryOperator::NotEquals => "!=",
        BinaryOperator::LessThan => "<",
        BinaryOperator::LessThanEquals => "<=",
        BinaryOperator::GreaterThan => ">",
        BinaryOperator::GreaterThanEquals => ">=",
        BinaryOperator::And => "&&",
        BinaryOperator::Or => "||",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Type;
    use crate::hir::*;

    fn int(v: i64) -> TypedExpression {
        TypedExpression::IntegerLiteral {
            value: v,
            type_: Type::Int,
            span: Span::new(0, 0),
        }
    }

    fn make_program(functions: Vec<TypedFunction>) -> TypedProgram {
        TypedProgram {
            app_metadata: None,
            functions,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn arithmetic_and_text() {
        let func = TypedFunction {
            name: "main".to_string(),
            name_span: Span::new(0, 0),
            parameters: Vec::new(),
            return_type: Type::Unit,
            body: TypedBlock {
                statements: vec![
                    TypedStatement::Let {
                        name: "x".to_string(),
                        type_: Type::Int,
                        initializer: TypedExpression::Binary {
                            left: Box::new(int(2)),
                            operator: BinaryOperator::Star,
                            right: Box::new(TypedExpression::Binary {
                                left: Box::new(int(3)),
                                operator: BinaryOperator::Plus,
                                right: Box::new(int(4)),
                                type_: Type::Int,
                                span: Span::new(0, 0),
                            }),
                            type_: Type::Int,
                            span: Span::new(0, 0),
                        },
                        span: Span::new(0, 0),
                    },
                    TypedStatement::Give {
                        value: int(10),
                        span: Span::new(0, 0),
                    },
                ],
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
        };
        let mut interp = Interpreter::new(make_program(vec![func]));
        let result = interp.run();
        assert!(result.is_ok());
    }

    #[test]
    fn division_by_zero_reported() {
        let func = TypedFunction {
            name: "main".to_string(),
            name_span: Span::new(0, 0),
            parameters: Vec::new(),
            return_type: Type::Unit,
            body: TypedBlock {
                statements: vec![TypedStatement::Expression {
                    expression: TypedExpression::Binary {
                        left: Box::new(int(1)),
                        operator: BinaryOperator::Slash,
                        right: Box::new(int(0)),
                        type_: Type::Int,
                        span: Span::new(0, 0),
                    },
                    span: Span::new(0, 0),
                }],
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
        };
        let mut interp = Interpreter::new(make_program(vec![func]));
        let result = interp.run();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, codes::DIVISION_BY_ZERO);
    }

    #[test]
    fn text_concatenation() {
        let func = TypedFunction {
            name: "main".to_string(),
            name_span: Span::new(0, 0),
            parameters: Vec::new(),
            return_type: Type::Unit,
            body: TypedBlock {
                statements: vec![TypedStatement::Expression {
                    expression: TypedExpression::Call {
                        callee: "print".to_string(),
                        arguments: vec![
                            TypedExpression::TextLiteral {
                                value: "yo, ".to_string(),
                                type_: Type::Text,
                                span: Span::new(0, 0),
                            },
                            TypedExpression::TextLiteral {
                                value: "world".to_string(),
                                type_: Type::Text,
                                span: Span::new(0, 0),
                            },
                        ],
                        return_type: Type::Unit,
                        span: Span::new(0, 0),
                    },
                    span: Span::new(0, 0),
                }],
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
        };
        let mut interp = Interpreter::new(make_program(vec![func]));
        // print takes exactly 1 argument; this should be a runtime error.
        let result = interp.run();
        assert!(result.is_err());
    }
}
