# Compiler handoff and work tracker

Updated: 2026-09-09. Shared carried-list borrowing is complete and passed the
compiler gate. No failing checks remain. Full v0.0.1 is incomplete.
[../STATUS.md](../STATUS.md) tracks the project; [../COMPILER.md](../COMPILER.md)
records the plan. Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Current compiler slice

`loans/transitive.rs::referenced` now applies `carried::storage` only to exclusive
Slot acquisition. `loans/emission_init.rs::emission_acquire` still attaches the
containing-slot Acquire event to every shared root. The existing state proof
requires the owner active and the full slot initialized, including empty lists.
The collection gate remains independently enforced for ExclusivePath and indexed
SetPath reservations. No AST/HIR, backend, runtime or dependency changes were needed.

Whole-list shared views, nested element/record-field projections and reborrows
retain canonical Slot/Field/Element sources and parent reference identity. Index
expressions execute once and check current initialized length. Returning index
access conflicts with replacement of its borrowed storage; canceled acquisition
preserves completed effects and skips unfinished checks/use.

Shared views survive inner restarts and alias scope exit while their result owner
lives. Owner completion, Leave and owner reset expire old sources; reinitializing
the same physical site cannot revive them. Overwriting an expired reference before
reading it retains the existing rule. Whole-list replacement remains E302 while a
whole/element view is live; disjoint fields and writes after final use remain valid.
Old reference copies and call-returned views retain their original lifetime bounds.

Known shared list headers coexist with supported local exclusive scalar sibling
loans under existing header certificates. Genuine call/input opacity and exclusive
descendants still reject reset frontiers. Exclusive acquisition of list-containing
carried storage remains B001, even for named scalar fields within those records;
indexed writes/reservations also remain gated. These are distinct next slices.
See [the contract](OWNERSHIP.md#shared-carried-list-borrows) and
[the example](examples/carried-list-borrows.mwy).

## Actual validation

- The whole-list restart regression first reproduced B001 from the blanket list
  gate. Only obsolete shared-borrow rejection cases were removed after focused
  source/native proof passed; genuine exclusive/indexed-write gates remain tested.
- Focused `cargo test --locked --manifest-path compiler/Cargo.toml --target
  x86_64-unknown-linux-gnu --target-dir compiler/target carried_list_borrows` passed
  10 source/proof groups and seven native groups, with native cases in both profiles.
- Graph evidence checks canonical containing-slot ownership, root Acquire without
  synthetic uses, exact field/element paths and copied parent reference identity.
  Missing Emit, inactive owner and acquisition after Complete all fail with B001
  at the original borrow span.
- Native checks cover retained views, nested parents/fields, current-length bounds,
  once-only/canceled index effects, owner reset, old copies, calls, empty lists,
  final use, disjoint writes, mixed headers and remaining primary rejections.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 622 library and 580 native Rust tests (1202 total), 16 tooling
  plus 4 compiler-harness Python tests, and 76 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities remain outside the full language gate.
- Local-link/whitespace checks passed; repository contracts also cover 23 catalog
  records and 7 schemas/6 examples. External links were not fetched. No editor,
  runtime/sanitizer, reference conformance fixture or dependency changes were needed;
  editor and separate runtime/sanitizer checks were not rerun.

## Prior capabilities and other areas

Carried reference-free lists retain whole initialized length/payload through inner
restarts. Shape limits remain 256 parts/32 levels; list construction retains capacity
65,536, layout 1 MiB and one-based initialized-length checks. Nullable/union/reference-
bearing/foundation/owning and top-level unit carried slots retain their gates.

Selected-slot mutability is independent of whole-binding replacement. `Proofs.mutable`
tracks replaceable roots and `Proofs.fields` mutable owned descendants; `variable`
drives snapshots/refinements without granting writes through shared references.
Plain carried-record exclusive scalar fields require every loan/descendant to end
before reset. Pointer syntax uses tight `&`/`&!`/`*`, immediate-field `.&`/`.&!`/`.*`
and grouping for complete targets.

Standalone documentation supports attachment, checked links/signatures, E801-E805,
`doc check`/`doc build`, local API pages and checked/opt-in examples. Net/HTTP/TLS
remain specified library work; module/type/I/O/task foundations and capability-typed
lifecycle implementation precede executable adapters.

## Architecture and proof boundaries

| Responsibility | Existing owner |
| --- | --- |
| Carried shape and remaining collection gate | `src/borrow/carried.rs` |
| Whole-slot initialized/active state and Acquire | `src/loans/emission_init.rs` |
| Shared direct/projected acquisition and parent identity | `src/loans/transitive.rs` |
| Element evaluation and initialized-length checks | `src/list.rs` |
| Exclusive indexed acquisition and indexed writes | `src/loans/elements.rs`, `src/loans/control.rs` |
| Shared-header certificates and exclusive reset frontier | `src/loans/restart_headers.rs`, `src/loans/exclusive_restarts.rs` |

## Still outside this compiler

The full module/package graph, generic specialization, captures, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Qualify exclusive named scalar fields within list-containing carried records
   before indexed access. Trace `borrow/carried.rs::storage`,
   `loans/transitive.rs::referenced` and
   `loans/exclusive_restarts.rs::exclusive_restart_source`; require initialized,
   active containing storage, exact field paths, selected-slot permission and no
   exclusive loan/descendant live across reset. Keep element acquisitions and
   indexed SetPath gated while this non-indexed path is qualified.
2. Cover list sibling replacement, nested fields, overlapping accesses, shared and
   exclusive children, last use, owner reset and Leave in source/native tests.
   Preserve shared-header certificates and genuine opacity; run the compiler gate.
3. Then investigate exclusive scalar list elements in `loans/elements.rs` and
   indexed writes in `loans/control.rs` independently. Each needs containing-slot
   initialization plus its own reservation, bounds, lifetime and frontier evidence.
   Broader carried shapes, owning cleanup and exclusive header carriage stay separate.
4. Continue module graphs/library foundations. Keep root/compiler STATUS current,
   commit cohesive validated work, and do not push or recreate STEP logs. Full
   release qualification remains open.
