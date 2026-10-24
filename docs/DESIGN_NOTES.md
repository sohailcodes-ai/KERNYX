# KERNYX Design Notes

Unresolved decisions and fundamental design questions. These are recorded
rather than pretending they have already been solved.

## Memory safety

Not yet defined. KERNYX 0.1 is an interpreter-backed language with garbage-free
but untracked memory; no ownership/borrowing model exists. Before claiming
memory safety, we must choose:

- Option A: Rust-like ownership/borrowing with a borrow checker.
- Option B: Region-based or arena-based memory with explicit lifetimes.
- Option C: Tracing garbage collection (simplest, but conflicts with
  "systems programming" identity).

No decision has been made. This is the primary blocker for milestone 0.3.

## Error handling

Currently: `give` returns values; runtime failures (division by zero, overflow,
undefined variable) are diagnostics that abort the program. There is no
recoverable error propagation mechanism (no `try`/`catch`, no `Result`-based
control flow). A precise error model (recoverable errors, propagation
semantics, panic vs. recoverable distinction) is a 0.3 concern.

## Effects

Not yet investigated. Explicit effect declarations for I/O, database, and other
side-effectful operations are a future research item. In 0.1, `print` is a
builtin with no effect tracking.

## Concurrency

Not yet defined. Open questions:
- Task lifetime and cancellation model.
- Shared mutable state vs. message passing.
- Scheduling (cooperative vs. preemptive).
- Error propagation across tasks.

`flow` is proposed syntax only; no semantics are implemented. These questions
must be answered before milestone 0.6.

## Native interoperability

Not yet planned. A practical ABI and foreign-function interface are a future
concern (post-0.5). No FFI exists in 0.1.

## Web support

Not yet defined. Questions:
- How are browser APIs called?
- How are native-only features rejected or isolated for WASM targets?

No WASM target exists in 0.1. Addressed at milestone 0.7.

## Database integration

Begin with libraries and typed query interfaces; do not promise a custom
database engine unless a concrete need is established. Addressed at milestone
0.8.

## Distinguishing computational capability

No specific differentiating feature has been identified. KERNYX's hypothesis is
that a single language can coherently serve systems programming, applications,
and high-concurrency software without splitting into separate language
identities. This is unproven. No "revolutionary" feature has been invented to
make the roadmap sound impressive; if a proposed feature has no clear
advantage, it is not included.
