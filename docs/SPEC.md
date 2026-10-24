# KERNYX Language Specification (Milestone 0.1)

This document describes the KERNYX language as implemented in version 0.1.0.
It describes only what is actually implemented; proposed future constructs are
listed separately and are not part of the working language.

## 1. Overview

KERNYX 0.1 is a small, statically typed, interpreted programming language.
Source files use the `.knx` extension. The implementation is a Rust program
(`kernyx`) that lexes, parses, type-checks, lowers to a typed intermediate
representation (HIR), and executes via an embedded interpreter.

A program must parse and pass static validation before any code executes.
Type errors prevent execution entirely.

## 2. Source files

- Extension: `.knx`
- Encoding: UTF-8 (decoded deliberately; invalid bytes are reported as lexer
  diagnostics, not panics).
- Line comments begin with `//` and run to the end of the line.

## 3. Lexical grammar

### Keywords

`app`, `fn`, `let`, `give`, `when`, `otherwise`, `return`, `Text`, `Int`,
`Float`, `Bool`, `Unit`, `true`, `false`.

The following are lexed as keywords but rejected by the parser as proposed
syntax (not implemented in 0.1): `flow`, `data`, `expose`.

### Literals

- Integer: a sequence of decimal digits, e.g. `42`, `-7` (unary minus).
- Float: digits with a decimal point, e.g. `3.14`.
- Text: double-quoted, UTF-8. Escapes: `\n`, `\t`, `\"`, `\\`. An unterminated
  string or invalid escape is a lexer diagnostic.
- Boolean: `true`, `false`.

### Punctuation and operators

`( ) { } , ; : -> + - * / % ! = == != < <= > >= && ||`

Whitespace (spaces, tabs, carriage returns, newlines) separates tokens.

Each token carries a byte span and a 1-indexed line/column location for
diagnostics.

## 4. Syntax

```
program      := app_decl? function*
app_decl     := "app" TEXT_LITERAL
function     := "fn" IDENT "(" params ")" ("->" type)? "{" block "}"
params       := param ("," param)*
param        := IDENT ":" type
block        := "{" statement* "}"
statement    := let_stmt | give_stmt | when_stmt | block_stmt | expr_stmt
let_stmt     := "let" IDENT (":" type)? "=" expression ";"
give_stmt    := "give" expression ";"
when_stmt    := "when" expression block ("otherwise" block)?
block_stmt   := "{" block "}"
expr_stmt    := expression ";"
expression   := binary (precedence climbing)
binary       := unary ((op unary)*) with precedence:
                ||  (1)
                &&  (2)
                == != (3)
                < <= > >= (4)
                + -  (5)
                * / % (6)
unary        := "-" unary | "+" unary | "!" unary | primary
primary      := INT | FLOAT | TEXT | BOOL | IDENT | "(" expression ")"
               | IDENT "(" arguments ")"
arguments    := expression ("," expression)*
```

### Type annotations

- Function parameters require explicit types: `name: Int`.
- `let` bindings infer their type from the initializer; an explicit annotation
  (`let x: Text = ...`) is optional and checked for agreement.
- Non-`main` functions must declare an explicit return type (`-> Type`).
- `main` may omit the return annotation and defaults to `Unit`.

### Return values

`give <expression>` returns a value from the current function. It is only
valid in functions with a non-`Unit` return type. A non-`Unit` function must
give a value on every code path; a missing return is a static error.

## 5. Types

| Type   | Meaning                        |
|--------|--------------------------------|
| Int    | signed 64-bit integer          |
| Float  | 64-bit floating-point number   |
| Bool   | boolean                        |
| Text   | UTF-8 string                   |
| Unit   | absence of a meaningful value  |

There is no implicit Int-to-Float conversion in 0.1. Types are nominal and
exactly equal for assignment and comparison purposes.

## 6. Scoping

- Function parameters are in scope throughout the function body.
- `let` bindings are block-scoped; bindings do not leak out of their block.
- Inner blocks may shadow outer bindings.
- `main` is the program entry point; a program without `main` is a static error.

## 7. Static type checking

Checked before execution:

- Undefined names (variables and functions).
- Duplicate top-level function names.
- Duplicate `let` bindings in the same scope.
- Function call argument count and types.
- Operator operand types (arithmetic requires matching Int/Float or Text+Text;
  comparisons require matching types; `&&`/`||` require Bool).
- `when` conditions must be `Bool`.
- `give` value type must match the function's declared return type.
- Non-`Unit` functions must return on all paths.
- `main` must exist.

A program with type errors never begins execution. The `check` command runs
only this phase.

## 8. Runtime

The validated HIR program is executed by a KERNYX interpreter (not Python,
JavaScript, shell, or generated source run by another language).

- `main` is invoked as the entry point.
- `print(value)` writes the value's textual rendering to stdout.
- Function calls, lexical scopes, bindings, `when`/`otherwise` branching,
  arithmetic, comparisons, booleans, text concatenation, and `give` returns are
  implemented.
- Integer arithmetic is checked: overflow and division by zero produce runtime
  diagnostics (non-zero exit) rather than wrapping or panicking.
- A documented recursion limit (1024 call frames) guards against unbounded
  recursion.

## 9. Errors and diagnostics

Every diagnostic includes: a stable error code (`KXnnnn`), a clear message,
file path, line/column, a source excerpt with a caret range indicator, and a
correction hint where one can be given without guessing.

- Compilation errors exit non-zero.
- Runtime errors exit non-zero.
- Ordinary user mistakes never produce stack traces or panics.
- Compiler internal errors are distinguishable from expected diagnostics.

Error code ranges:
- 1001–1007: lexer
- 101–107: parser
- 201–212: type checker
- 301–306: runtime

## 10. CLI

```
kernyx run <file>.knx    parse, type-check, and execute
kernyx check <file>.knx  parse and type-check only
kernyx --version         print version
kernyx help              print usage
```

Exit codes: 0 success, 1 compilation/runtime error, 2 usage error.

## 11. Limitations (0.1)

The following are explicitly NOT implemented in this milestone:

- Async execution, `await`, concurrency (`flow` is proposed syntax only).
- Generics, modules, data types (`data` is proposed syntax only).
- Explicitly published APIs (`expose` is proposed syntax only).
- Mutable bindings (`let` is immutable only).
- Native machine-code generation, bytecode VM, WASM.
- Networking, HTTP, database libraries.
- Ownership/borrowing, memory-safety model.
- Package manager, formatter, language server, debugger.

See docs/ROADMAP.md for the planned sequence and exit criteria.
