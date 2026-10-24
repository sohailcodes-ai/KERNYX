# KERNYX Semantics

Status: 0.1 frozen; post-0.1 areas marked unresolved

This document specifies the meaning of KERNYX programs: typing, scoping, evaluation,
errors, and the current memory/resource/effect/concurrency strategy. It is kept
synchronized with the implementation in `src/`.

## 1. Types (0.1)

| Type   | Meaning                        | Size/representation |
|--------|--------------------------------|---------------------|
| Int    | signed 64-bit integer          | i64                 |
| Float  | 64-bit IEEE-754 floating-point | f64                 |
| Bool   | boolean                        | bool                |
| Text   | UTF-8 string                   | String              |
| Unit   | absence of a meaningful value  | zero-width value    |

There is no implicit Int-to-Float conversion in 0.1. Types are nominal and exactly equal
for assignment and comparison. No generics, no algebraic data types, no option/null type
in 0.1.

## 2. Scoping

- Function parameters are in scope throughout the function body.
- `let` bindings are block-scoped; bindings do not leak out of their block.
- Inner blocks may shadow outer bindings.
- `main` is the program entry point; a program without `main` is a static error
  (KX0212).
- Top-level function names must be unique (KX0103 / KX0201).

## 3. Declaration and typing rules

- Function parameters require explicit types (`name: Int`).
- `let` bindings infer their type from the initializer; an explicit annotation
  (`let x: Text = ...`) is optional and checked for agreement (KX0203).
- Non-`main` functions must declare an explicit return type (KX0206).
- `main` may omit the return annotation and defaults to `Unit`.
- `print` is a builtin taking exactly one argument; returns `Unit`.
- Function calls are checked for argument count and types (KX0204, KX0205).
- Operators are type-checked: arithmetic requires matching Int/Float, or Text+Text for
  concatenation; comparisons require matching types; `&&`/`||` require Bool operands
  (KX0211).
- `when` conditions must be `Bool` (KX0209).
- `give` is only valid in functions with a non-`Unit` return type (KX0208); the `give`
  value's type must match the declared return type.
- A non-`Unit` function must give a value on every code path (KX0207); a missing return
  is a static error, never a silently manufactured value.
- Undefined names (variables and functions) are rejected (KX0202).
- Duplicate `let` bindings in the same scope are rejected (KX0210).

Return-path analysis: a `give` statement makes its enclosing block definitely return. A
`when` definitely returns only if both branches (or the single branch when there is no
`otherwise`) definitely return. Statements after a definite return remain parsed so
further errors can be collected, but they are unreachable at runtime. This milestone
does not reject or warn about unreachable statements explicitly; they are simply
unreachable. This is documented here rather than left implicit.

## 4. Evaluation

Validated HIR programs are executed by a tree-walk interpreter in Rust (not Python,
JavaScript, shell, or generated host-language source).

- `main` is invoked as the entry point with no arguments.
- `print(value)` writes the value's textual rendering to stdout.
- Integer arithmetic is checked: overflow and division-by-zero produce runtime
  diagnostics (non-zero exit) rather than wrapping (KX0301, KX0302).
- Floating-point division by zero produces a runtime diagnostic (KX0301).
- A recursion limit of 1024 call frames guards against unbounded recursion (KX0303).
- User-program errors are distinguished from compiler internal errors; ordinary
  malformed input never panics.

Evaluation order: operands of binary expressions are evaluated left to right; function
arguments are evaluated left to right before the call.

## 5. Memory and resource model (0.1)

Not yet defined. KERNYX 0.1 is an interpreter-backed language with no ownership,
borrowing, regions, or garbage collector. Memory is managed by the Rust interpreter's
own heap; no static memory-safety guarantees are made.

Decision deferred to milestone 0.3. Options under consideration (see
docs/research/COMPETITORS.md): Rust-like ownership/borrowing, region-based management,
tracing GC, ARC, hybrid. No claim of memory safety is made until the chosen model is
implemented and tested.

## 6. Effects and capabilities (0.1)

Not yet implemented. `print` is a builtin with no effect tracking. Effect/capability
declarations for I/O, database, network, unsafe operations, and target-specific APIs are
a future research item (milestone 0.5).

## 7. Concurrency (0.1)

Not implemented. No `async`, `await`, `flow`, tasks, cancellation, or shared mutable
state. `flow` is proposed syntax only and is rejected by the 0.1 parser. Concurrency
semantics must be specified before implementation (milestone 0.6).

## 8. Execution boundaries and targets (0.1)

Single target only: the KERNYX interpreter. No native compilation, no WASM, no browser
support. Cross-target constraints are not yet expressed or checked. Planned for
milestones 0.4 (native backend decision) and 0.7 (WASM/browser).

## 9. Error behavior

- Compilation errors (lexer KX1001–1005, parser KX0101–0107, type checker KX0201–0212)
  exit non-zero; the program never executes.
- Runtime errors (KX0301–KX0306) exit non-zero.
- Diagnostics include a stable error code, file path, line/column, source excerpt with
  caret range, and a correction hint where one can be given without guessing.
- Compiler internal errors are distinguishable from expected user diagnostics.
- Ordinary malformed input never produces a stack trace or panic.

## 10. Unresolved areas (explicit)

- Memory/resource model: ownership vs. region vs. GC undecided.
- Error handling: no recoverable error propagation (`try`/`catch`, `Result`-based
  control flow) defined.
- Effects: no explicit effect system defined.
- Concurrency: task lifetime, cancellation, scheduling, shared state undecided.
- Native interop: no ABI/FFI planned.
- Web: browser API calling and native-only feature isolation undecided.
- Distinguishing computational thesis: unproven (see docs/THESIS.md).
