# KERNYX Competitor and Design Review

Status: bounded synthesis
Researched: 2026-10-25 (web search + language documentation)
Accessed sources:
- https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html (Rust ownership)
- https://google.github.io/comprehensive-rust/memory-management/approaches.html (memory management approaches)
- https://export.arxiv.org/pdf/2301.02308v1.pdf (Usability of Advanced Type Systems: Rust as a Case Study, Ferdowsi, 2023)
- https://wasi.dev/ (WASI overview)
- https://webassembly.org/docs/non-web/ (WebAssembly non-web embeddings)
- https://github.com/WebAssembly/WASI/ (WASI repository)

This is a deliberately bounded review of language families relevant to KERNYX's thesis.
It is not a feature checklist of every language.

## Comparison

| Problem/use case | Existing solution(s) | Strengths | Trade-offs / limitations | KERNYX hypothesis | Evidence still required |
|---|---|---|---|---|---|
| Systems control + memory safety without GC | Rust (ownership/borrowing) | Compile-time memory + thread safety, zero-cost, mature ecosystem | Steep learning curve; ownership model is a barrier for ordinary application tasks (Ferdowsi 2023); complex type system | A simpler boundary-aware model can give useful static guarantees without full ownership complexity | A demonstrated program where the boundary model catches a real cross-target bug with less notation than Rust |
| Backend productivity, simple deployment | Go | Fast compile times, GC, simple concurrency, easy deployment | No browser target; no explicit memory control; boundaries implicit | Same language across native/server/browser with explicit boundaries | A cross-target module that type-checks identically for native and WASM |
| Browser + backend, dynamic/gradual typing | TypeScript/JavaScript | Huge ecosystem, first-class browser support | Not a systems language; no native executable with explicit memory control; runtime type erasure | Static types that survive to native and browser targets without erasure at boundaries | A shared data contract validated across a native-to-WASM boundary by the compiler |
| Multi-target compilation | Haxe, Dart, Kotlin Multiplatform | One source to several targets | Boundaries erased/flattened; remote calls look like local calls; serialization implicit; capability differences surface at runtime | Target/capability constraints explicit in the type system; local vs. remote vs. platform calls distinguishable | A program where calling an unavailable platform API is a compile error, not a runtime failure |
| Portable sandboxed binary | WebAssembly + WASI | Run once, anywhere; capability-based sandbox (host grants authority); runs browser to cloud | Wasm historically "only talks in numbers"; rich cross-language interop needs component model/glue; no native memory model of its own | KERNYX source compiles to Wasm/WASI with explicit platform API rejection/isolation | A generated Wasm module runs outside the browser under WASI and fails safely on denied capabilities |
| Effects / capability tracking | OCaml, Haskell, ML family, effect systems | Precise effect typing, algebraic data types, pattern matching | Niche adoption; effects can become annotation clutter | Effects declared where they add measurable value; not ambient | A program where an effectful operation cannot be secretly invoked from a "pure" context |
| Native + systems interoperability | Zig, C/C++ | Explicit control, compile-time facilities, C ABI | Manual memory management; C/C++ unsoundness; Zig still maturing | Explicit control where needed, safe-by-default elsewhere | Not yet applicable (pre-0.3) |

## Synthesis

The component ideas (ownership, regions, effects, multi-target compilation, WASI capability
sandboxing) are proven in isolation. The research review did not find a mainstream
language that combines them into one coherent, explicitly boundary-aware model targeting
a single developer across native, server, and browser. That combination remains
unproven as an advantage, not a claim.

Key cautions from the research:
- Ownership/borrowing gives strong guarantees but has documented usability costs; do not
  adopt it merely because it sounds systems-oriented.
- Multi-target compilation tends to erase boundaries, which is the exact problem KERNYX
  wants to make explicit; this is a known failure mode to design around.
- WASI gives a capability-based sandbox but Wasm modules start with no ambient authority;
  KERNYX must express which capabilities a module requires and reject unavailable ones at
  compile time where the model allows.

## Sources

- Rust ownership: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
- Memory management approaches: https://google.github.io/comprehensive-rust/memory-management/approaches.html
- Ferdowsi, "The Usability of Advanced Type Systems: Rust as a Case Study" (2023): https://export.arxiv.org/pdf/2301.02308v1.pdf
- WASI: https://wasi.dev/ and https://github.com/WebAssembly/WASI/
- WebAssembly non-web embeddings: https://webassembly.org/docs/non-web/
