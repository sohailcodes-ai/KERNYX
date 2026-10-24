# KERNYX Status

Status: 0.1 complete

This document lists implemented, partial, planned, rejected, and blocked features. It is
the authoritative capability matrix and must be kept consistent with the README and
docs/SPEC.md.

## Implemented and verified (0.1)

- Lexer: identifiers, keywords, punctuation, operators, integer/float/boolean/text
  literals; whitespace and `//` line comments; text escapes `\n`, `\t`, `\"`, `\\`;
  source byte spans and line/column locations; invalid character and unterminated
  string diagnostics (no panics).
- Parser: optional `app "name"` metadata, top-level function declarations, typed
  parameters, optional declared return type (`main` defaults to `Unit`), function bodies,
  immutable `let` bindings (optional explicit annotation), `give` statements,
  expression statements, `when`/`otherwise`, nested blocks, parentheses and calls, unary
  and binary expressions with documented precedence, equality/ordering/arithmetic/boolean
  operators, duplicate function rejection, malformed declaration rejection, sync-point
  error recovery.
- Type checker: `Int`, `Float`, `Bool`, `Text`, `Unit`; let type inference; explicit
  parameter types; non-`main` explicit return type; `main` defaults to `Unit`; call,
  operator, condition, and return-value checking; undefined name, incompatible type,
  invalid call, duplicate declaration, and invalid `give` rejection; lexical scoping with
  no leakage and shadowing; return-path analysis for non-`Unit` functions.
- HIR: typed program representation (stable frontend/backend boundary).
- Interpreter: `main` entry point, `print` builtin to stdout, function calls, lexical
  scopes/bindings, branching, arithmetic, comparisons, booleans, text concatenation,
  `give` returns; checked integer overflow and division-by-zero; 1024-frame recursion
  limit; user errors distinguished from internal errors; no panics on malformed input.
- CLI: `kernyx run <file>`, `kernyx check <file>`, `kernyx --version`, `kernyx help`;
  exit codes 0/1/2; diagnostics to stderr, program output to stdout; nonexistent file
  handling.
- Tests: 27 unit + 9 integration tests; all pass; `cargo fmt --check` and
  `cargo clippy --all-targets -- -D warnings` clean.
- Examples: `examples/hello.knx`, `examples/arithmetic.knx`, `examples/functions.knx`,
  `examples/errors/{bad_type,bad_return,bad_condition,bad_arg,div_by_zero}.knx`.
- Docs: SPEC.md, ROADMAP.md, DESIGN_NOTES.md, THESIS.md, SYNTAX_DESIGN.md, GRAMMAR.md,
  SEMANTICS.md, ARCHITECTURE.md, STATUS.md, research/COMPETITORS.md, README.md.

## Partial / experimental

None in 0.1.

## Planned / not implemented

- Async execution, `await`, concurrency (`flow` is proposed syntax only).
- Generics, modules, declared data types (`data` is proposed syntax only).
- Explicit API publishing (`expose` is proposed syntax only).
- Mutable bindings (`let mut`).
- Native machine-code generation, bytecode VM, WASM.
- Networking, HTTP, databases.
- Ownership/borrowing; any memory-safety model.
- Package manager, formatter, language server, debugger.
- Recoverable error propagation; effect/capability system.
- Cross-target execution-boundary checking.

## Rejected in 0.1 (proposed syntax, not working)

`flow`, `data`, `expose`, `return` are lexed as keywords and rejected by the parser with
a diagnostic stating they are proposed syntax.

## Blocked

None.

## Capability matrix (mirrored in README)

| Capability | State | Notes |
|---|---|---|
| Lexing / parsing | implemented | 0.1 |
| Static type checking | implemented | 0.1 |
| Interpretation / execution | implemented | 0.1 |
| Diagnostics with source locations | implemented | 0.1 |
| CLI (run/check/version/help) | implemented | 0.1 |
| Native compilation | planned | milestone 0.4 |
| Memory-safety model | undecided | milestone 0.3 |
| Effects / capabilities | planned | milestone 0.5 |
| Concurrency | planned | milestone 0.6 |
| WebAssembly / browser | planned | milestone 0.7 |
| Networking / HTTP / databases | planned | milestone 0.8 |
| Package / formatter / language server | planned | milestone 0.9+ |
