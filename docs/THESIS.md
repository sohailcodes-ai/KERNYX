# KERNYX Design Thesis

Status: draft
Audience: KERNYX maintainers and contributors

## 1. First target user

The first target user is a solo or small-team application developer who is tired of
splitting a single product across several languages: a native CLI or desktop tool in
Rust/C++, a backend service in Go or TypeScript, and browser UI in TypeScript. That
developer pays integration, serialization, type-mismatch, and context-switching costs at
every boundary between those environments.

KERNYX's first target user wants one language, one type model, and one toolchain across
native, server, and browser targets — without pretending those environments have
identical capabilities.

## 2. Problem

Existing languages cover parts of this ground, but none make cross-target execution
boundaries an explicit, statically checked part of program design:

- Rust gives systems control and safety but is not ergonomic for rapid application or
  web development; its ownership model is a steep learning curve for ordinary tasks.
- Go is productive for backends but has no serious browser story.
- TypeScript dominates the browser and backends but is not a systems language and has no
  native executable story with explicit memory control.
- Multi-target languages (Haxe, Dart, Kotlin) compile to several targets but generally
  erase or flatten the boundary between them: a remote call looks like a local call,
  serialization is implicit, and platform capability differences surface as runtime
  failures rather than compile-time errors.

The painful, repeatable task is writing and maintaining the same data model and logic
across native, server, and browser code while manually keeping serialization, capability
availability, and error propagation in sync.

## 3. Design hypothesis (execution-boundary-aware programming)

KERNYX's candidate differentiator is to treat execution boundaries — native process,
server, browser/WASM, data service — as explicit, statically checked parts of program
design, rather than implicit implementation details.

Concretely, the language could:

- Attach explicit target/capability constraints to modules, functions, or dependencies.
- Model shared data contracts that are validated across compilation targets.
- Distinguish local calls, remote calls, and platform API calls in the type system so
  cross-boundary failure, latency, serialization, and trust are visible to the
  programmer.
- Keep effects and capabilities (I/O, database, network, unsafe memory, target-specific
  APIs) explicit rather than ambient.

This is a HYPOTHESIS, not a proven invention. Research (see docs/research/COMPETITORS.md)
shows the component parts exist in isolation (Rust ownership, WASI capability sandboxing,
effect systems in OCaml/ML/Haskell, multi-target compilation in Haxe/Dart/Kotlin), but
no mainstream language combines them into one coherent, explicitly boundary-aware model
aimed at a single developer across native/server/browser.

If the hypothesis is weak, it will be refined or replaced based on the milestone 3
demonstration (see docs/ROADMAP.md).

## 4. What KERNYX guarantees statically (0.1)

For the current milestone, the guarantees are deliberately small:

- Every program that executes has passed parsing and static type checking.
- Type errors prevent execution entirely.
- Function calls are checked for argument count and types.
- `when` conditions must be `Bool`; `give` values must match the declared return type.
- Non-`Unit` functions must return on all code paths.
- Integer arithmetic is checked at runtime (overflow/division-by-zero are diagnostics,
  not silent wrappers).
- Bindings are block-scoped and immutable; no implicit Int-to-Float coercion.

Memory safety, concurrency safety, effects, and cross-target boundaries are NOT yet
guaranteed. They are explicitly listed as planned/not implemented in docs/STATUS.md and
the README capability matrix.

## 5. Validation criteria

The thesis is validated only by an end-to-end demonstration (milestone 3) that shows:

1. A single KERNYX source module type-checks for two different execution targets
   (e.g. native and WASM) without modification.
2. A cross-boundary call (local vs. remote vs. platform API) is visibly distinct in the
   program and its failure mode is declared/checked statically where the model allows.
3. The shared data contract across the boundary is validated by the compiler, not by
   ad-hoc runtime checks.

Until that demonstration exists, claims about KERNYX's uniqueness or advantage are
marked as unproven hypotheses.

## 6. Trade-offs accepted

- Verbosity at boundaries: explicit target/capability annotations cost more to write
  than implicit calls, but they make failures local and diagnosable at compile time.
- Slower initial velocity: a small, strictly typed core ships before broad application
  libraries.
- No claim to outperform Rust, Go, or TypeScript on their home turf; the claim is
  boundary-coherence, not raw speed.

## 7. Unanswered questions

- Which concrete capability/target syntax survives contact with real programs?
- How are cross-boundary failures serialized and surfaced without annotation clutter?
- What is the smallest demo that proves the thesis rather than merely showcasing syntax?

These are recorded in docs/DESIGN_NOTES.md and docs/DECISIONS/.
