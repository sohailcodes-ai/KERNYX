# KERNYX
> every language has a trade-off. i got tired of picking.

so naturally, i decided to make another programming language. excellent career decision.

KERNYX aims to bring systems-level control and application-level ergonomics into one
general-purpose language.

not another syntax-highlighting project. not a wrapper pretending to be a compiler. an
actual language, built one working piece at a time.

## What works today (0.1)

KERNYX 0.1 is a complete, small vertical slice: a lexer, recursive-descent parser,
statically typed checker, typed intermediate representation (HIR), and a tree-walk
interpreter — all in Rust, standard library only.

The following program runs:

```
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

Output:

```
yo, world
```

CLI:

- `kernyx run examples/hello.knx` — parse, type-check, execute.
- `kernyx check examples/hello.knx` — type-check without executing.
- `kernyx --version`
- `kernyx help`

A program with type errors never begins execution. Diagnostics carry stable error
codes (`KX0001`-style), file/line/column, source excerpts, and hints.

## What does not work yet

- Async / concurrency (`flow` is proposed syntax only).
- Generics, modules, declared data types (`data` is proposed syntax only).
- Explicit API publishing (`expose` is proposed syntax only).
- Mutable bindings (`let` is immutable).
- Native machine code, bytecode VM, WASM.
- Networking, HTTP, databases.
- Ownership / borrowing / any memory-safety model.
- Package manager, formatter, language server, debugger.

KERNYX is not production-ready, not memory-safe, not native-compiled, not
concurrent, and not suitable for web development yet. These are goals, not
features.

## Rust toolchain

Rust edition 2021. No external dependencies for the first milestone — standard
library only.

## Commands

```
cargo build              # build the kernyx CLI
cargo run -- run examples/hello.knx
cargo run -- check examples/hello.knx
cargo test               # unit + integration tests
cargo fmt --check        # formatting
cargo clippy --all-targets -- -D warnings  # lint
```

## Architecture

```
source text
  -> lexer (tokens with byte spans)
  -> parser (AST)
  -> name resolution + type checking (sema)
  -> typed HIR
  -> interpreter (runtime)
```

The HIR is the stable boundary between the frontend and backends; future native
and WASM backends consume HIR rather than re-parsing source. The parser is
independent of the runtime; type checking is separate from evaluation.

## Current limitations

See docs/SPEC.md (§11) and docs/DESIGN_NOTES.md for the exhaustive list, plus
explicit statements about what is not claimed.

## Roadmap

See docs/ROADMAP.md. Milestones 0.2–0.9+ are tentative and not delivery
promises.

## Contribution status

This is a solo, early-stage language project. The first milestone is complete
and tested. There is no formal contribution process yet; the codebase is
intentionally small and readable so it can be picked up incrementally.
