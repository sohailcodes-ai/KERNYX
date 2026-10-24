# KERNYX

> A general-purpose language exploring one coherent model for systems and application software.

KERNYX is a programming language project built in Rust. The long-term goal is to combine systems-level control with application-level ergonomics, while making execution boundaries—native, server, browser, and data services—explicit instead of leaving them hidden in framework conventions.

That is the thesis, not a claim that the problem is solved. KERNYX is early-stage, and every capability below is labeled according to what is actually implemented today.

## Current release: 0.1

KERNYX 0.1 is a small, runnable language implementation with a complete frontend-to-execution path:

- Lexer with source spans
- Recursive-descent parser
- Static type checker
- Typed high-level intermediate representation (HIR)
- Tree-walk interpreter
- Command-line interface for running and checking programs
- Compiler diagnostics with stable error codes, source locations, excerpts, and hints

The compiler and runtime are implemented in Rust 2021 using the standard library only. The 0.1 implementation is an interpreter; it does **not** compile KERNYX to native machine code.

## Try it

You need a Rust toolchain installed.

```powershell
cargo build
cargo run -- run examples/hello.knx
```

Expected output:

```text
yo, world
```

Check a program without executing it:

```powershell
cargo run -- check examples/hello.knx
```

Run the project's checks:

```powershell
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## A KERNYX program

```rust
app "hello"

fn greet(name: Text) -> Text {
    give "yo, " + name
}

fn main() {
    let user = greet("world")

    when user != "" {
        print(user)
    } otherwise {
        print("nah, nothing here")
    }
}
```

This example uses syntax implemented in 0.1: `app`, `fn`, `let`, `give`, `when`, `otherwise`, function calls, text concatenation, comparisons, and `print`.

A program that fails static type checking is rejected before execution. The current interpreter also checks operations such as integer overflow and division by zero at runtime.

## What exists—and what doesn't

| Area | 0.1 status |
|---|---|
| Lexing and parsing | Implemented |
| Static type checking | Implemented |
| Typed HIR | Implemented |
| Interpreter and basic execution | Implemented |
| CLI: `run`, `check`, version and help | Implemented |
| Source-located diagnostics with KX codes and hints | Implemented |
| Native machine-code backend | Not implemented |
| Ownership, borrowing or a memory-safety model | Not implemented; design undecided |
| Effects and capability checking | Not implemented |
| Async and concurrency | Not implemented |
| WebAssembly and browser targets | Not implemented |
| HTTP, networking and database libraries | Not implemented |
| Generics, modules and declared data types | Not implemented in 0.1 |
| Package manager, formatter, language server and debugger | Not implemented |
| REALTALK diagnostic personality and explain-code UX | Proposed; not part of the reported 0.1 implementation |

`flow`, `data`, and `expose` are design ideas, not working language features in 0.1. The language is not currently production-ready, memory-safe by a defined language guarantee, natively compiled, concurrent, or ready for browser/backend development.

## Design direction

KERNYX is investigating **execution-boundary-aware programming**: the idea that target environments, capabilities, and contracts between parts of an application should be represented clearly and checked where possible.

The goal is to reduce the friction between systems code, backend services, browser code, and data-facing components without pretending that these environments have identical permissions or runtime capabilities.

This thesis is still unproven. Its value must be demonstrated through implemented features and measurable examples—not through syntax sketches or roadmap promises.

### Architectural shape

```text
source text
    ↓
lexer → parser → name resolution and type checking
    ↓
typed HIR
    ↓
interpreter today; future backends only when implemented
```

HIR is intended to be the stable boundary between the frontend and future execution backends. The parser does not depend on the runtime, and type checking is separate from evaluation.

## Roadmap

The roadmap is tentative. Milestones can change as implementation and experiments reveal what is useful.

| Milestone | Direction |
|---|---|
| 0.2 | Richer core language: declared data types, pattern matching, and multi-file modules |
| 0.3 | Define the memory and resource model |
| 0.4 | Investigate and implement a native compilation backend |
| 0.5 | Define and implement effects and capability checks |
| 0.6 | Specify async and concurrency semantics |
| 0.7 | Investigate WebAssembly and browser execution |
| 0.8 | Networking, HTTP, and database support |
| 0.9+ | Package management, formatter, language server, debugger, and ecosystem tooling |

Developer-experience work—including richer offline compiler diagnostics under the proposed **REALTALK** direction—should improve the actual compiler as it grows. REALTALK is a planned feature, not a capability to rely on in 0.1.

See [`docs/ROADMAP.md`](docs/ROADMAP.md), [`docs/THESIS.md`](docs/THESIS.md), [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), and [`docs/STATUS.md`](docs/STATUS.md) for details and decisions.

## Project principles

- **Ship working slices.** A feature is implemented when it runs and is tested—not when it appears in a syntax example.
- **Make trade-offs explicit.** Memory management, concurrency, target support, and effects require real design decisions.
- **Keep diagnostics honest.** Explain what the compiler knows; don't invent causes or fixes.
- **Prefer reproducible behavior.** Keep builds and tests deterministic where possible.
- **Document the limits.** Planned capabilities are not release features.

## Contributing and status

KERNYX is an early-stage solo language project. The 0.1 vertical slice is implemented; the larger language design is actively being explored. Start with [`CONTRIBUTING.md`](CONTRIBUTING.md) and the design documents before proposing architectural changes.

The ambition is broad. The implementation status is deliberately specific.
