# KERNYX Roadmap

These are tentative milestones, not delivery promises. The sequence may change
as implementation evidence justifies it; changes are documented here with
rationale.

## Milestone 0.1 (complete)

Lexer, parser, static type checker, typed HIR, interpreter, diagnostics, tests,
CLI.

Exit criteria:
- `examples/hello.knx` parses, type-checks, executes and prints exactly
  `yo, world`.
- `kernyx check` detects type errors without executing.
- Test suite exercises successful execution and negative cases.
- Diagnostics contain useful source locations.
- `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`
  all pass.

## Milestone 0.2 — richer core language

Goals:
- Better type inference (e.g. infer non-main return types where possible).
- Explicit mutable bindings (`let mut`).
- Richer declared data types and exhaustive pattern matching.
- Module semantics (import/export across files).

Exit criteria:
- A program using pattern matching over declared data types compiles and runs.
- Mutable rebinding works; immutable-by-default is preserved.
- Modules split across multiple `.knx` files resolve and type-check together.

Rationale for ordering: inference and mutability are small, high-value
correctness improvements; modules are needed before a standard library can be
built.

## Milestone 0.3 — ownership and resource management

Goals:
- Define and implement an explicit ownership/borrowing model (or an alternative
  memory-safety model).
- More complete type-system rules (e.g. drop/check resource cleanup).

Exit criteria:
- A documented memory-safety model exists and is enforced by the checker.
- No use-after-free or double-free patterns type-check.

Rationale: memory safety must be defined before claiming it; this milestone
addresses the fundamental design question in docs/DESIGN_NOTES.md.

## Milestone 0.4 — bytecode VM and profiling

Goals:
- A bytecode VM as a backend alternative to the tree-walk interpreter.
- Profiling to measure VM vs. interpreter performance.

Exit criteria:
- The VM executes all 0.1 programs with identical observable behavior.
- Measured performance data justifies (or rejects) the VM as the default
  runtime.

Rationale: the tree-walk interpreter is a correct starting point but a VM is a
natural step toward a native backend; measurement decides the trade-off.

## Milestone 0.5 — native backend strategy

Goals:
- A justified native compilation strategy.
- Real generated executable artifacts with correct error handling.

Exit criteria:
- A native backend produces a runnable executable from a KERNYX program.
- Runtime errors from generated executables carry source locations.

Rationale: native compilation is a stated goal but must not be faked; the
strategy (C backend, LLVM, Cranelift, etc.) is chosen based on evidence from
0.4.

## Milestone 0.6 — concurrency semantics

Goals:
- Task lifetime, cancellation, shared mutable state, scheduling, error
  propagation.
- Async/await.

Exit criteria:
- A concurrent program runs with deterministic error propagation.
- Cancellation and backpressure have defined behavior.

Rationale: concurrency semantics must be precise before implementation; the
design is recorded in docs/DESIGN_NOTES.md.

## Milestone 0.7 — WebAssembly and browser interop

Goals:
- WebAssembly compilation.
- Browser API interop; native-only features rejected or isolated for WASM.

Exit criteria:
- A KERNYX program compiles to WASM and runs in a browser host.
- Native-only features produce clear target-incompatibility diagnostics.

## Milestone 0.8 — networking and standard library

Goals:
- Networking, HTTP APIs, database drivers in a tested standard library.
- Typed query interfaces (no custom database engine unless a concrete need is
  established).

Exit criteria:
- A standard-library HTTP client and database driver are tested against real
  services.

## Milestone 0.9+ — ecosystem tooling

Goals:
- Package management, formatter, language server, debugger.

Exit criteria:
- `kernyx fmt`, `kernyx pkg`, and a language server are usable end to end.
