# Contributing to KERNYX

KERNYX is an early-stage, solo language project. There is no formal contribution process
yet; this document describes how to build, test, format, and run the compiler so the
codebase can be advanced incrementally.

## Prerequisites

- Rust toolchain, edition 2021 (verified with rustc/cargo 1.98.1).
- No external dependencies for the first milestone; the project uses the Rust standard
  library only.

## Commands

```
cargo build              # build the kernyx CLI
cargo run -- run examples/hello.knx   # run a program
cargo run -- check examples/hello.knx # type-check without executing
cargo test               # unit + integration tests
cargo fmt --check        # formatting (run `cargo fmt` to fix)
cargo clippy --all-targets -- -D warnings  # lint
```

## Repository layout

```
src/           compiler implementation (lexer, parser, sema, hir, runtime, cli)
examples/      .knx programs used by integration tests
examples/errors/ intentionally invalid programs checked by tests
tests/         integration tests (full programs + exit codes)
docs/          specification, roadmap, design notes, syntax/semantics/architecture,
               research, status, decision records
```

## Adding a change

1. Inspect the current state; preserve useful existing work.
2. Implement one complete, testable slice at a time.
3. Add tests for both valid and invalid behavior; diagnostics must assert source
   locations/properties, not merely "did not crash."
4. Run `cargo fmt --check`, `cargo test`, and
   `cargo clippy --all-targets -- -D warnings`.
5. Verify against `examples/hello.knx` (expects output `yo, world`) and at least one
   invalid program (expects a non-zero exit and a diagnostic).
6. Update docs/SPEC.md and docs/STATUS.md to stay synchronized with behavior.

## Do not

- Claim production readiness, memory safety, native compilation, concurrency, or web
  support until implemented and demonstrated.
- Fabricate benchmarks or diagnostics.
- Add dependencies without a concrete need and a recorded trade-off.
- Commit or push without authorization.
