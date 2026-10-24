# ADR 0001 — Defer memory/resource model decision

Status: accepted
Date: 2026-10-25
Deciders: KERNYX principal engineer

## Context

KERNYX's long-term vision includes systems programming with explicit control over
memory/resources. The 0.1 milestone is an interpreter-backed typed frontend with no
ownership, borrowing, regions, or garbage collector. Before claiming memory safety, a
model must be chosen and implemented.

Options considered:
- Rust-like ownership/borrowing: strong compile-time memory + thread safety without GC,
  but documented steep learning curve (Ferdowsi 2023); may be overkill for ordinary
  application tasks.
- Region-based management: static deallocation at scope end; simpler than full ownership
  but adds region annotations/inference complexity.
- Tracing GC: simplest ergonomics, but conflicts with predictable low-level control and
  latency constraints.
- ARC / reference counting: ergonomic but imposes runtime overhead and programmer burden
  for cycles.
- Hybrid (safe-by-default GC heap + explicit regions/unsafe for systems code): balances
  ergonomics and control but increases implementation complexity.

## Decision

Defer the memory/resource model to milestone 0.3. Do not implement or claim any memory
safety in 0.1. KERNYX 0.1 makes no memory-safety guarantees; memory is managed by the
Rust interpreter's heap.

## Rationale

The first milestone must prove a complete, small language (lex, parse, type-check,
execute) before layering a memory model on top. Choosing ownership/borrowing merely
because it sounds systems-oriented, or GC without accounting for latency, would be an
unsupported decision. The 0.3 milestone will select a model based on the documented
trade-offs and implement it with positive and negative tests.

## Consequences

- docs/DESIGN_NOTES.md, docs/SEMANTICS.md, docs/STATUS.md, and the README all state that
  memory safety is not implemented or claimed in 0.1.
- No `unsafe`, borrow checker, region, or GC code exists in 0.1.
- The 0.3 milestone acceptance criteria include a documented memory-safety model enforced
  by the checker.

## Related

- docs/research/COMPETITORS.md
- docs/DESIGN_NOTES.md (memory safety section)
- docs/ROADMAP.md (milestone 0.3)
