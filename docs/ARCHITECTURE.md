# KERNYX Architecture

Status: 0.1 implemented; future backends planned

## 1. Pipeline

```
source text
  -> source manager (Source: path + text + line-start table for location lookups)
  -> lexer (Token stream with byte spans + 1-indexed line/column)
  -> parser (recursive-descent, AST with explicit enums/structs)
  -> name resolution + static type checking (sema)
  -> typed HIR (typed intermediate representation)
  -> interpreter (tree-walk execution)
```

Each stage produces diagnostics with source spans. Spans survive from lexing through type
checking so errors carry accurate file/line/column and source excerpts.

## 2. Module boundaries and responsibilities

- `src/source.rs` — `Source` and `Span`; byte-range tracking, line/column location of
  any byte offset, line-text excerpt.
- `src/diagnostic.rs` — `Diagnostic`, `Diagnostics`, `Formatter`; stable `KXnnnn`
  error codes, severity, source excerpt, caret indicator, hint.
- `src/lexer/` — tokenization (`token.rs` for `Token`/`TokenKind`/`LexError`; `mod.rs`
  for the lexer state machine). Reports invalid characters and unterminated strings as
  diagnostics, not panics.
- `src/parser/` — recursive-descent parser producing an untyped AST; precedence-climbing
  expression parsing; error-recovery via sync points so multiple errors are reported.
- `src/ast.rs` — untyped AST (`Program`, `FunctionDeclaration`, `Statement`,
  `Expression`, `Type`, operators).
- `src/sema/mod.rs` — name resolution, lexical scopes, type checking, return-path
  analysis, lowering to HIR.
- `src/hir.rs` — typed HIR (`TypedProgram`, `TypedFunction`, `TypedStatement`,
  `TypedExpression`); the stable frontend/backend boundary.
- `src/runtime/mod.rs` — tree-walk interpreter over typed HIR; builtin `print`; checked
  integer arithmetic; division-by-zero and overflow diagnostics; recursion limit.
- `src/cli.rs` — `run`, `check`, `--version`, `help`; exit codes 0/1/2; diagnostics
  written to stderr, program output to stdout.
- `src/main.rs` — module declarations and entry point.

## 3. Architectural invariants

- The parser is independent of the runtime.
- Type checking is separate from evaluation; the interpreter consumes validated, typed
  HIR.
- HIR is the stable boundary: future native and WASM backends consume HIR rather than
  re-parsing source.
- No `eval` or generated host-language code; KERNYX is not translated to Python,
  JavaScript, or Rust and executed.
- Parser errors, type errors, runtime program errors, and compiler bugs are
  distinguishable.
- Ordinary malformed input does not crash the compiler.
- Source spans survive through the frontend for accurate diagnostics.
- No silent coercions or undefined behavior without explicit specification
  (no implicit Int-to-Float in 0.1).
- No dependency is added without a concrete job; the 0.1 implementation uses the Rust
  standard library only.

## 4. Future backend plan

- Milestone 0.4: select a native backend strategy (bytecode VM vs. Cranelift vs. LLVM
  vs. direct object code) based on measured trade-offs; produce a real executable.
- Milestone 0.7: WebAssembly compilation with WASI; browser interop with explicit
  rejection/isolation of native-only features.
- Backends consume HIR; the frontend is not rewritten for each backend.

## 5. What is NOT part of the architecture (0.1)

- No bytecode VM (planned 0.4).
- No native code generation (planned 0.4).
- No WASM backend (planned 0.7).
- No package manager, formatter, or language server (planned 0.9+).
