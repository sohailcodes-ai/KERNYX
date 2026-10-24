use crate::ast::{
    BinaryOperator, Expression, FunctionDeclaration, Program, Statement, Type, UnaryOperator,
};
use crate::diagnostic::{Diagnostic, Diagnostics};
use crate::hir::{
    BinaryOperator as HBinaryOperator, TypedBlock, TypedExpression, TypedFunction, TypedParameter,
    TypedProgram, TypedStatement, UnaryOperator as HUnaryOperator,
};
use crate::source::{Source, Span};
use std::collections::HashMap;

mod codes {
    use crate::diagnostic::DiagnosticCode;
    pub const UNDEFINED_FUNCTION: DiagnosticCode = DiagnosticCode::from_u32(201);
    pub const UNDEFINED_VARIABLE: DiagnosticCode = DiagnosticCode::from_u32(202);
    pub const TYPE_MISMATCH: DiagnosticCode = DiagnosticCode::from_u32(203);
    pub const WRONG_ARGUMENT_COUNT: DiagnosticCode = DiagnosticCode::from_u32(204);
    pub const ARGUMENT_TYPE_MISMATCH: DiagnosticCode = DiagnosticCode::from_u32(205);
    pub const MISSING_RETURN_TYPE: DiagnosticCode = DiagnosticCode::from_u32(206);
    pub const MISSING_RETURN_VALUE: DiagnosticCode = DiagnosticCode::from_u32(207);
    pub const INVALID_GIVE: DiagnosticCode = DiagnosticCode::from_u32(208);
    pub const NON_BOOLEAN_CONDITION: DiagnosticCode = DiagnosticCode::from_u32(209);
    pub const DUPLICATE_VARIABLE: DiagnosticCode = DiagnosticCode::from_u32(210);
    pub const INVALID_OPERAND_TYPES: DiagnosticCode = DiagnosticCode::from_u32(211);
    pub const MAIN_NOT_FOUND: DiagnosticCode = DiagnosticCode::from_u32(212);
}

const BUILTIN_PRINT: &str = "print";
const MAIN_NAME: &str = "main";

/// Signature of a declared function, collected before body checking so that
/// functions may call one another in any order.
#[derive(Clone)]
struct FunctionSignature {
    name: String,
    parameters: Vec<(String, Type)>,
    return_type: Type,
    span: Span,
}

/// One lexical scope of variable bindings.
struct Scope {
    bindings: HashMap<String, (Type, Span)>,
}

impl Scope {
    fn new() -> Scope {
        Scope {
            bindings: HashMap::new(),
        }
    }

    fn declare(&mut self, name: String, type_: Type, span: Span) -> Result<(), Diagnostic> {
        if self.bindings.contains_key(&name) {
            return Err(Diagnostic::error(
                codes::DUPLICATE_VARIABLE,
                format!("duplicate binding `{}`", name),
                span,
            ));
        }
        self.bindings.insert(name, (type_, span));
        Ok(())
    }

    fn lookup(&self, name: &str) -> Option<Type> {
        self.bindings.get(name).map(|(ty, _)| ty.clone())
    }
}

/// Type-checks a parsed program and lowers it to a typed HIR program.
pub struct Checker<'a> {
    source: &'a Source,
    diagnostics: Diagnostics,
    signatures: HashMap<String, FunctionSignature>,
    scopes: Vec<Scope>,
    current_return: Option<Type>,
}

impl<'a> Checker<'a> {
    pub fn new(source: &'a Source) -> Checker<'a> {
        Checker {
            source,
            diagnostics: Diagnostics::new(),
            signatures: HashMap::new(),
            scopes: Vec::new(),
            current_return: None,
        }
    }

    pub fn check(mut self, program: Program) -> (Option<TypedProgram>, Diagnostics) {
        self.collect_signatures(&program);

        if !self.signatures.contains_key(MAIN_NAME) {
            self.diagnostics.items.push(Diagnostic::error(
                codes::MAIN_NOT_FOUND,
                "no `main` function found; every program needs a `main` entry point",
                program.span,
            ));
        }

        let mut typed_functions = Vec::new();
        for func in &program.functions {
            if let Some(typed) = self.check_function(func) {
                typed_functions.push(typed);
            }
        }

        if self.diagnostics.has_errors() {
            (None, self.diagnostics)
        } else {
            let span = typed_functions
                .first()
                .map(|f| f.span)
                .unwrap_or_else(|| program.span);
            let typed = TypedProgram {
                app_metadata: program.app_metadata,
                functions: typed_functions,
                span,
            };
            (Some(typed), self.diagnostics)
        }
    }

    fn collect_signatures(&mut self, program: &Program) {
        for func in &program.functions {
            let return_type = match &func.return_type {
                Some(ty) => ty.clone(),
                None if func.name == MAIN_NAME => Type::Unit,
                None => {
                    self.diagnostics.items.push(Diagnostic::error(
                        codes::MISSING_RETURN_TYPE,
                        format!(
                            "non-main function `{}` must declare an explicit return type",
                            func.name
                        ),
                        func.name_span,
                    ));
                    Type::Unit
                }
            };

            let parameters: Vec<(String, Type)> = func
                .parameters
                .iter()
                .map(|p| (p.name.clone(), p.type_annotation.clone()))
                .collect();

            if self
                .signatures
                .insert(
                    func.name.clone(),
                    FunctionSignature {
                        name: func.name.clone(),
                        parameters,
                        return_type,
                        span: func.span,
                    },
                )
                .is_some()
            {
                // Duplicate already reported by parser; keep one diagnostic.
                self.diagnostics.items.push(Diagnostic::error(
                    codes::UNDEFINED_FUNCTION,
                    format!("duplicate function definition `{}`", func.name),
                    func.name_span,
                ));
            }
        }
    }

    fn check_function(&mut self, func: &FunctionDeclaration) -> Option<TypedFunction> {
        let signature = self.signatures.get(&func.name).cloned().unwrap();
        self.current_return = Some(signature.return_type.clone());
        self.scopes.push(Scope::new());

        // Declare parameters in the function scope.
        for param in &func.parameters {
            self.declare_local(&param.name, param.type_annotation.clone(), param.name_span)
                .ok();
        }

        let body = self.check_block(&func.body);
        self.scopes.pop();

        let return_type = signature.return_type.clone();
        let needs_return = return_type != Type::Unit;
        let body = match body {
            Some(b) => b,
            None => {
                self.current_return = None;
                return None;
            }
        };
        let definite_return = self.block_definitely_returns(&body);
        if needs_return && !definite_return {
            self.diagnostics.items.push(Diagnostic::error(
                codes::MISSING_RETURN_VALUE,
                format!(
                    "function `{}` returns `{}` but not all code paths give a value",
                    func.name, return_type
                ),
                func.span,
            ));
            self.current_return = None;
            return None;
        }

        self.current_return = None;
        Some(TypedFunction {
            name: func.name.clone(),
            name_span: func.name_span,
            parameters: func
                .parameters
                .iter()
                .map(|p| TypedParameter {
                    name: p.name.clone(),
                    type_: p.type_annotation.clone(),
                    span: p.span,
                })
                .collect(),
            return_type,
            body,
            span: func.span,
        })
    }

    fn check_block(&mut self, block: &crate::ast::Block) -> Option<TypedBlock> {
        self.scopes.push(Scope::new());
        let mut statements = Vec::new();
        let mut block_definitely_returns = false;

        for stmt in &block.statements {
            let (typed, definitely_returns) = self.check_statement(stmt);
            if let Some(typed) = typed {
                statements.push(typed);
            }
            if definitely_returns {
                block_definitely_returns = true;
            }
            // Statements after a definite return are unreachable; warn but keep parsing.
            if block_definitely_returns {
                // Continue to collect further errors but mark rest unreachable.
            }
        }

        self.scopes.pop();
        Some(TypedBlock {
            statements,
            span: block.span,
        })
    }

    fn check_statement(&mut self, stmt: &Statement) -> (Option<TypedStatement>, bool) {
        match stmt {
            Statement::Let {
                name,
                name_span,
                type_annotation,
                type_span: _,
                initializer,
                span,
            } => {
                let init_typed = self.check_expression(initializer);
                let _init_type = init_typed.as_ref().map(|e| e.type_()).unwrap_or(Type::Unit);
                let declared = type_annotation.as_ref().cloned();
                let final_type = match (&declared, init_typed.as_ref()) {
                    (Some(declared), Some(init)) => {
                        if !declared.assignable_to(&init.type_()) {
                            self.diagnostics.items.push(
                                Diagnostic::error(
                                    codes::TYPE_MISMATCH,
                                    format!(
                                        "declared type `{}` does not match initializer type `{}`",
                                        declared,
                                        init.type_()
                                    ),
                                    initializer.span(),
                                )
                                .with_hint("remove the annotation or change the initializer"),
                            );
                        }
                        declared.clone()
                    }
                    (Some(declared), None) => declared.clone(),
                    (None, Some(init)) => init.type_(),
                    (None, None) => Type::Unit,
                };

                self.declare_local(name, final_type.clone(), *name_span)
                    .ok();

                (
                    Some(TypedStatement::Let {
                        name: name.clone(),
                        type_: final_type,
                        initializer: init_typed.unwrap_or(TypedExpression::IntegerLiteral {
                            value: 0,
                            type_: Type::Int,
                            span: *span,
                        }),
                        span: *span,
                    }),
                    false,
                )
            }
            Statement::Expression { expression, span } => {
                let typed = self.check_expression(expression);
                (
                    typed.map(|expr| TypedStatement::Expression {
                        expression: expr,
                        span: *span,
                    }),
                    false,
                )
            }
            Statement::Block { block, span } => {
                let block_span = block.span;
                let typed = self.check_block(block);
                let typed = typed.unwrap_or_else(|| empty_block(block_span));
                let definitely_returns = self.block_definitely_returns(&typed);
                (
                    Some(TypedStatement::Block {
                        block: typed,
                        span: *span,
                    }),
                    definitely_returns,
                )
            }
            Statement::Give { value, span } => {
                let expected = self.current_return.clone().unwrap_or(Type::Unit);
                let typed = self.check_expression(value);
                let typed = match typed {
                    Some(expr) => {
                        if expected != Type::Unit && expr.type_() != expected {
                            self.diagnostics.items.push(Diagnostic::error(
                                codes::INVALID_GIVE,
                                format!(
                                    "give value of type `{}` does not match return type `{}`",
                                    expr.type_(),
                                    expected
                                ),
                                expr.span(),
                            ));
                        }
                        expr
                    }
                    None => TypedExpression::IntegerLiteral {
                        value: 0,
                        type_: expected.clone(),
                        span: *span,
                    },
                };
                if expected == Type::Unit {
                    self.diagnostics.items.push(
                        Diagnostic::error(
                            codes::INVALID_GIVE,
                            "`give` used in a function that returns `Unit`",
                            *span,
                        )
                        .with_hint("give is only valid in functions with a non-Unit return type"),
                    );
                }
                (
                    Some(TypedStatement::Give {
                        value: typed,
                        span: *span,
                    }),
                    true,
                )
            }
            Statement::When {
                condition,
                then_branch,
                otherwise_branch,
                span,
            } => {
                let cond_typed = self.check_expression(condition);
                let cond_type = cond_typed.as_ref().map(|e| e.type_()).unwrap_or(Type::Bool);
                if cond_type != Type::Bool {
                    self.diagnostics.items.push(Diagnostic::error(
                        codes::NON_BOOLEAN_CONDITION,
                        format!("when condition must be `Bool`, found `{}`", cond_type),
                        condition.span(),
                    ));
                }

                let then_typed = self.check_block(then_branch);
                let otherwise_typed: Option<TypedBlock> = match otherwise_branch {
                    Some(b) => self.check_block(b),
                    None => None,
                };

                let definitely_returns = then_typed
                    .as_ref()
                    .is_some_and(|b| self.block_definitely_returns(b))
                    && otherwise_typed
                        .as_ref()
                        .is_none_or(|b| self.block_definitely_returns(b));

                (
                    Some(TypedStatement::When {
                        condition: cond_typed.unwrap_or(TypedExpression::BooleanLiteral {
                            value: false,
                            type_: Type::Bool,
                            span: condition.span(),
                        }),
                        then_branch: then_typed.unwrap_or_else(|| empty_block(then_branch.span)),
                        otherwise_branch: otherwise_typed,
                        span: *span,
                    }),
                    definitely_returns,
                )
            }
        }
    }

    fn check_expression(&mut self, expr: &Expression) -> Option<TypedExpression> {
        match expr {
            Expression::IntegerLiteral { value, span } => Some(TypedExpression::IntegerLiteral {
                value: *value,
                type_: Type::Int,
                span: *span,
            }),
            Expression::FloatLiteral { value, span } => Some(TypedExpression::FloatLiteral {
                value: *value,
                type_: Type::Float,
                span: *span,
            }),
            Expression::TextLiteral { value, span } => Some(TypedExpression::TextLiteral {
                value: value.clone(),
                type_: Type::Text,
                span: *span,
            }),
            Expression::BooleanLiteral { value, span } => Some(TypedExpression::BooleanLiteral {
                value: *value,
                type_: Type::Bool,
                span: *span,
            }),
            Expression::Identifier { name, span } => {
                let type_ = self.lookup_variable(name);
                match type_ {
                    Some(ty) => Some(TypedExpression::Var {
                        name: name.clone(),
                        type_: ty,
                        span: *span,
                    }),
                    None => {
                        // Maybe it's a function being used without call? Error.
                        self.diagnostics.items.push(Diagnostic::error(
                            codes::UNDEFINED_VARIABLE,
                            format!("undefined variable `{}`", name),
                            *span,
                        ));
                        None
                    }
                }
            }
            Expression::Unary {
                operator,
                operand,
                span,
            } => {
                let inner = self.check_expression(operand)?;
                let (op, result_type) = match operator {
                    UnaryOperator::Minus => match inner.type_() {
                        Type::Int => (HUnaryOperator::Minus, Type::Int),
                        Type::Float => (HUnaryOperator::Minus, Type::Float),
                        other => {
                            self.diagnostics.items.push(Diagnostic::error(
                                codes::INVALID_OPERAND_TYPES,
                                format!("cannot negate `{}`", other),
                                inner.span(),
                            ));
                            (HUnaryOperator::Minus, Type::Int)
                        }
                    },
                    UnaryOperator::Plus => match inner.type_() {
                        Type::Int => (HUnaryOperator::Plus, Type::Int),
                        Type::Float => (HUnaryOperator::Plus, Type::Float),
                        other => {
                            self.diagnostics.items.push(Diagnostic::error(
                                codes::INVALID_OPERAND_TYPES,
                                format!("cannot apply unary + to `{}`", other),
                                inner.span(),
                            ));
                            (HUnaryOperator::Plus, Type::Int)
                        }
                    },
                    UnaryOperator::Not => match inner.type_() {
                        Type::Bool => (HUnaryOperator::Not, Type::Bool),
                        other => {
                            self.diagnostics.items.push(Diagnostic::error(
                                codes::INVALID_OPERAND_TYPES,
                                format!("cannot apply `!` to `{}`", other),
                                inner.span(),
                            ));
                            (HUnaryOperator::Not, Type::Bool)
                        }
                    },
                };
                Some(TypedExpression::Unary {
                    operator: op,
                    operand: Box::new(inner),
                    type_: result_type,
                    span: *span,
                })
            }
            Expression::Binary {
                left,
                operator,
                right,
                span,
            } => {
                let left = self.check_expression(left)?;
                let right = self.check_expression(right)?;
                let (op, result_type) = self.check_binary_types(&left, operator, &right)?;
                Some(TypedExpression::Binary {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    type_: result_type,
                    span: *span,
                })
            }
            Expression::Call {
                callee,
                arguments,
                span,
            } => {
                let callee_name = match callee.as_ref() {
                    Expression::Identifier { name, .. } => name.clone(),
                    _ => {
                        self.diagnostics.items.push(Diagnostic::error(
                            codes::UNDEFINED_FUNCTION,
                            "only direct function names can be called",
                            callee.span(),
                        ));
                        return None;
                    }
                };

                let typed_args: Vec<TypedExpression> = arguments
                    .iter()
                    .filter_map(|a| self.check_expression(a))
                    .collect();

                if callee_name == BUILTIN_PRINT {
                    if typed_args.len() != 1 {
                        self.diagnostics.items.push(Diagnostic::error(
                            codes::WRONG_ARGUMENT_COUNT,
                            format!("print takes exactly 1 argument, got {}", typed_args.len()),
                            *span,
                        ));
                    }
                    return Some(TypedExpression::Call {
                        callee: BUILTIN_PRINT.to_string(),
                        arguments: typed_args,
                        return_type: Type::Unit,
                        span: *span,
                    });
                }

                let signature = self.signatures.get(&callee_name);
                match signature {
                    Some(sig) => {
                        if typed_args.len() != sig.parameters.len() {
                            self.diagnostics.items.push(Diagnostic::error(
                                codes::WRONG_ARGUMENT_COUNT,
                                format!(
                                    "function `{}` expects {} arguments, got {}",
                                    callee_name,
                                    sig.parameters.len(),
                                    typed_args.len()
                                ),
                                *span,
                            ));
                        }
                        for (i, (arg, (param_name, param_type))) in
                            typed_args.iter().zip(sig.parameters.iter()).enumerate()
                        {
                            if arg.type_() != *param_type {
                                self.diagnostics.items.push(Diagnostic::error(
                                    codes::ARGUMENT_TYPE_MISMATCH,
                                    format!(
                                        "argument {} to `{}`: expected `{}`, found `{}`",
                                        i + 1,
                                        callee_name,
                                        param_type,
                                        arg.type_()
                                    ),
                                    arg.span(),
                                ));
                            }
                            let _ = param_name;
                        }
                        Some(TypedExpression::Call {
                            callee: callee_name.clone(),
                            arguments: typed_args,
                            return_type: sig.return_type.clone(),
                            span: *span,
                        })
                    }
                    None => {
                        self.diagnostics.items.push(Diagnostic::error(
                            codes::UNDEFINED_FUNCTION,
                            format!("undefined function `{}`", callee_name),
                            callee.span(),
                        ));
                        None
                    }
                }
            }
        }
    }

    fn check_binary_types(
        &mut self,
        left: &TypedExpression,
        operator: &BinaryOperator,
        right: &TypedExpression,
    ) -> Option<(HBinaryOperator, Type)> {
        let lt = left.type_();
        let rt = right.type_();
        let op = match operator {
            BinaryOperator::Plus => HBinaryOperator::Plus,
            BinaryOperator::Minus => HBinaryOperator::Minus,
            BinaryOperator::Star => HBinaryOperator::Star,
            BinaryOperator::Slash => HBinaryOperator::Slash,
            BinaryOperator::Percent => HBinaryOperator::Percent,
            BinaryOperator::Equals => HBinaryOperator::Equals,
            BinaryOperator::NotEquals => HBinaryOperator::NotEquals,
            BinaryOperator::LessThan => HBinaryOperator::LessThan,
            BinaryOperator::LessThanEquals => HBinaryOperator::LessThanEquals,
            BinaryOperator::GreaterThan => HBinaryOperator::GreaterThan,
            BinaryOperator::GreaterThanEquals => HBinaryOperator::GreaterThanEquals,
            BinaryOperator::And => HBinaryOperator::And,
            BinaryOperator::Or => HBinaryOperator::Or,
        };

        let arithmetic = matches!(
            operator,
            BinaryOperator::Plus
                | BinaryOperator::Minus
                | BinaryOperator::Star
                | BinaryOperator::Slash
                | BinaryOperator::Percent
        );
        let comparison = matches!(
            operator,
            BinaryOperator::Equals
                | BinaryOperator::NotEquals
                | BinaryOperator::LessThan
                | BinaryOperator::LessThanEquals
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterThanEquals
        );
        let logical = matches!(operator, BinaryOperator::And | BinaryOperator::Or);

        if arithmetic {
            if lt == Type::Int && rt == Type::Int {
                return Some((op, Type::Int));
            }
            if lt == Type::Float && rt == Type::Float {
                return Some((op, Type::Float));
            }
            if lt == Type::Text && rt == Type::Text && operator == &BinaryOperator::Plus {
                return Some((op, Type::Text));
            }
            self.diagnostics.items.push(Diagnostic::error(
                codes::INVALID_OPERAND_TYPES,
                format!(
                    "cannot apply `{}` to `{}` and `{}`",
                    operator_symbol(operator),
                    lt,
                    rt
                ),
                left.span(),
            ));
            return Some((op, Type::Int));
        }

        if comparison {
            if lt == rt {
                return Some((op, Type::Bool));
            }
            self.diagnostics.items.push(Diagnostic::error(
                codes::TYPE_MISMATCH,
                format!("cannot compare `{}` and `{}`", lt, rt),
                left.span(),
            ));
            return Some((op, Type::Bool));
        }

        if logical {
            if lt == Type::Bool && rt == Type::Bool {
                return Some((op, Type::Bool));
            }
            self.diagnostics.items.push(Diagnostic::error(
                codes::NON_BOOLEAN_CONDITION,
                format!(
                    "`&&` and `||` require `Bool` operands, found `{}` and `{}`",
                    lt, rt
                ),
                left.span(),
            ));
            return Some((op, Type::Bool));
        }

        Some((op, Type::Unit))
    }

    fn declare_local(&mut self, name: &str, type_: Type, span: Span) -> Result<(), Diagnostic> {
        self.scopes
            .last_mut()
            .expect("no active scope")
            .declare(name.to_string(), type_, span)
    }

    fn lookup_variable(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.lookup(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    fn block_definitely_returns(&self, block: &TypedBlock) -> bool {
        if block.statements.is_empty() {
            return false;
        }
        match &block.statements[block.statements.len() - 1] {
            TypedStatement::Give { .. } => true,
            TypedStatement::When {
                then_branch,
                otherwise_branch,
                ..
            } => {
                self.block_definitely_returns(then_branch)
                    && otherwise_branch
                        .as_ref()
                        .is_some_and(|b| self.block_definitely_returns(b))
            }
            TypedStatement::Block { block, .. } => self.block_definitely_returns(block),
            _ => false,
        }
    }
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

fn empty_block(span: Span) -> TypedBlock {
    TypedBlock {
        statements: Vec::new(),
        span,
    }
}

/// Entry point: run name resolution and type checking on a parsed program.
pub fn check_program(source: &Source, program: Program) -> (Option<TypedProgram>, Diagnostics) {
    Checker::new(source).check(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::source::Source;

    fn check(text: &str) -> (Option<TypedProgram>, Diagnostics) {
        let source = Source::new("test.knx", text.to_string());
        let (tokens, lex_errors) = Lexer::new(&source).lex();
        let (program, parse_errors) = Parser::new(tokens).parse();
        let mut diagnostics = Diagnostics::new();
        let (typed, check_diag) = check_program(&source, program);
        diagnostics.items.extend(lex_errors);
        diagnostics.items.extend(parse_errors);
        diagnostics.items.extend(check_diag.items);
        (typed, diagnostics)
    }

    #[test]
    fn infers_let_and_checks_call() {
        let text = r#"
fn add(a: Int, b: Int) -> Int {
    give a + b
}
fn main() -> Unit {
    let x = add(1, 2)
}
"#;
        let (typed, diag) = check(text);
        assert!(!diag.has_errors(), "unexpected errors: {:?}", diag.items);
        assert!(typed.is_some());
    }

    #[test]
    fn rejects_type_mismatch() {
        let text = r#"
fn main() -> Unit {
    let x: Text = 42
}
"#;
        let (_, diag) = check(text);
        assert!(diag.has_errors());
        assert!(diag.items.iter().any(|d| d.code == codes::TYPE_MISMATCH));
    }

    #[test]
    fn rejects_undefined_variable() {
        let text = r#"
fn main() -> Unit {
    let y = nope
}
"#;
        let (_, diag) = check(text);
        assert!(diag.has_errors());
        assert!(diag
            .items
            .iter()
            .any(|d| d.code == codes::UNDEFINED_VARIABLE));
    }

    #[test]
    fn rejects_non_boolean_condition() {
        let text = r#"
fn main() -> Unit {
    when 42 {
    }
}
"#;
        let (_, diag) = check(text);
        assert!(diag.has_errors());
        assert!(diag
            .items
            .iter()
            .any(|d| d.code == codes::NON_BOOLEAN_CONDITION));
    }

    #[test]
    fn main_defaults_to_unit() {
        let text = r#"
fn main() {
}
"#;
        let (typed, diag) = check(text);
        assert!(!diag.has_errors());
        let main = typed
            .unwrap()
            .functions
            .into_iter()
            .find(|f| f.name == "main")
            .unwrap();
        assert_eq!(main.return_type, Type::Unit);
    }

    #[test]
    fn non_main_requires_return_type() {
        let text = r#"
fn greet() {
}
fn main() -> Unit {
}
"#;
        let (_, diag) = check(text);
        assert!(diag.has_errors());
        assert!(diag
            .items
            .iter()
            .any(|d| d.code == codes::MISSING_RETURN_TYPE));
    }

    #[test]
    fn missing_return_value_rejected() {
        let text = r#"
fn f() -> Int {
}
fn main() -> Unit {
}
"#;
        let (_, diag) = check(text);
        assert!(diag.has_errors());
        assert!(diag
            .items
            .iter()
            .any(|d| d.code == codes::MISSING_RETURN_VALUE));
    }

    #[test]
    fn scopes_do_not_leak() {
        let text = r#"
fn main() -> Unit {
    {
        let x = 1
    }
    let y = x
}
"#;
        let (_, diag) = check(text);
        assert!(diag.has_errors());
        assert!(diag
            .items
            .iter()
            .any(|d| d.code == codes::UNDEFINED_VARIABLE));
    }

    #[test]
    fn shadowing_allowed_in_inner_scope() {
        let text = r#"
fn main() -> Unit {
    let x = 1
    {
        let x = 2
        let y = x
    }
    let z = x
}
"#;
        let (typed, diag) = check(text);
        assert!(!diag.has_errors(), "unexpected: {:?}", diag.items);
        assert!(typed.is_some());
    }
}
