# Compiler handoff and work tracker

Updated: 2026-09-09. Binding/field mutability is refactored and passed the compiler
gate. No failing checks remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Current compiler slice

`:` prevents replacing its binding or field slot; it does not freeze mutable fields
inside an owned value. `check/mutation.rs::record_field` returns the selected field's
flag alongside its type/index. Field selection replaces the current permission;
list indexing inherits it. The final slot must be mutable for assignment or exclusive
acquisition. Root assignment still requires `:=`. Shared/reference-derived access
retains existing permission and capability restrictions. Function/dispatch owned
copies retain field permissions without changing their caller's original value.

`Proofs.mutable` remains whole-root replacement permission. `Type::has_mutable_fields`
identifies mutable owned descendants without following references; `Checker::local`
records those IDs in `Proofs.fields`. `Proofs::variable` combines both for refinement
invalidation and branch/restart state tracking. This keeps immutable bindings with
changing reference fields out of frozen constructor/tag assumptions. Old copies,
nullable activity, public input bounds and owner expiry retain their evidence.

Fixed reference/allocator-bearing record aliases with mutable fields now use existing
versioned origin/bound snapshots, publication and refresh machinery. A preservation
case caught an unintended rejection of an unchanged allocator-bound alias; qualified
record snapshots fix it. Scalar allocator alias limitations remain unchanged.
`result_slot` resolves backing independently of replacement permission; actual writes
are authorized at their selected path, and declared alias/root flags still match
HIR/completed backing. Field updates cannot extend emitted-owner lifetimes.

Both origin and loan analyses retain selected-slot checks for indexed exclusive
paths; carried scalar-field source qualification follows the final field flag.
Whole-binding replacement, immutable list-element slots and live shared/exclusive
conflicts remain rejected. The [reference rule](../docs/reference/values-and-blocks.md#mutability)
and [example](examples/binding-fields.mwy) show the intended inside/outside behavior.
No backend, runtime, dependency, pointer grammar or reference conformance fixture
changes were needed.

## Actual validation

- The user's inside/outside mutation example first reproduced E305 before edits.
- Eight new source groups and eight native groups pass. Coverage includes immutable
  roots/enclosing records, selected field/list-slot permissions, reference branches
  and restart versions, emitted aliases, nullable activity, invalidated predicates,
  allocator bounds, carried records, function/dispatch copies and captured stores.
- Existing ancestor-mutability rejection fixtures now include positive debug/release
  execution where the selected field is mutable. Immutable selected slots, malformed
  alias metadata, whole-root replacement, E301/E302/E303 and shared-access restrictions
  remain checked. Capability gates were not relabeled as conformance rejections.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 612 library and 573 native Rust tests (1185 total), 16 tooling
  plus 4 compiler-harness Python tests, and 75 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities remain outside the full language gate.
- Local-link/whitespace checks passed. Repository contracts cover 23 catalog records
  and 7 schemas/6 examples; external links were not fetched. Editor and the separate
  runtime/sanitizer gate were not rerun; their code is unchanged.

## Prior capabilities and other areas

Carried fixed-capacity reference-free lists retain whole initialized length/payload
through inner restarts. Copies, replacement/addition, nested shapes, owner reset,
Leave and partial panics are qualified. Shape limits remain 256 parts/32 levels;
list construction retains capacity 65,536, layout 1 MiB and one-based length checks.
`carried::storage` still gates original list-containing carried storage at ordinary
Acquire, exclusive indexed acquisition and indexed SetPath reservations. Plain
sibling slots, copies and completed results keep their existing rules.

Plain carried records retain shared projections and local exclusive scalar-field
loans; every exclusive loan and descendant must end before restart. Shared headers,
conservative call/input ancestry and old-copy loans remain independent requirements.
The pointer syntax migration is complete: tight `&`/`&!`/`*`, immediate-field
`.&`/`.&!`/`.*` and grouping for complete targets.

Standalone documentation supports attachment, checked links/signatures, E801-E805,
`doc check`/`doc build`, local API pages and checked/opt-in examples. Net/HTTP/TLS
remain specified library work; module/type/I/O/task foundations, capability types
and lifecycle implementation precede executable adapters.

## Architecture and proof boundaries

| Responsibility | Existing owner |
| --- | --- |
| Field lookup and selected replacement permission | `src/check/mutation.rs`, `src/check/references.rs`, `src/check/indexed.rs` |
| Changing owned fields versus replaceable roots | `src/hir.rs`, `src/check/names.rs`, `src/borrow/state.rs` |
| Refinements and current reference/bound snapshots | `src/check/refinement.rs`, `src/borrow/`, `src/loans/` |
| Carried shape and collection access gate | `src/borrow/carried.rs` |
| Whole-slot initialized/active state and Acquire | `src/loans/emission_init.rs` |
| Shared direct/projected acquisition | `src/loans/transitive.rs` |
| Exclusive indexed acquisition and indexed writes | `src/loans/elements.rs`, `src/loans/control.rs` |

## Still outside this compiler

The full module/package graph, generic specialization, captures, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Keep selected-slot mutability distinct from changing-value tracking in subsequent
   work. Use `Proofs.mutable` for whole-root permission and `variable` for snapshots/
   refinements. Do not propagate mutable-field metadata through shared references.
2. Resume shared carried-list storage/element borrowing in
   `loans/transitive.rs::referenced`, `loans/emission_init.rs::emission_acquire`,
   `list.rs::element_borrow` and `borrow/carried.rs::storage`. Separate shared
   acquisition from exclusive/indexed-write gates and keep the latter restricted.
3. Require active whole-slot initialization, current length bounds, precise sources,
   parent authority and owner expiry. Cover inner restarts, nested fields/lists,
   old copies, last use, reset/Leave and conflicts in source/native tests, then run
   `tools/verify.py --compiler` before relaxing the shared-acquisition gate.
4. Broader carried shapes, owning cleanup and exclusive header carriage stay separate.
   Continue module graphs/library foundations, use migrated pointer syntax and keep
   root/compiler STATUS current. Commit cohesive validated work; do not push or
   recreate STEP logs. Full release qualification remains open.
