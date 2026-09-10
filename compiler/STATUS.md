# Compiler handoff and work tracker

Updated: 2026-09-10. Local exclusive carried scalar elements are complete and
passed the compiler gate. No failing checks remain.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the project;
[../COMPILER.md](../COMPILER.md) records the plan. Keep this handoff current;
Git holds history. Do not recreate STEP logs.

## Current compiler slice

`loans/elements.rs::exclusive_path` now permits carried scalar list elements and
mixed named-field/index paths using the existing independent owner/type proof.
Containing-slot Acquire occurs before the owner read/reservations and at completed
acquisition. The entire slot must be initialized and its owner active, even when
an index later cancels. No runtime flags or extra payload reads are introduced.

`exclusive_restart_source` now traverses Element only through List and Field only
through Record. Indexing inherits the slot permission; named fields supply their
own. Canonical owner/root/view, scalar Boolean/integer/float leaves and no live
exclusive or opaque ancestry across reset remain required. Mutable scalar fields
inside immutable owned list slots are supported without permitting element replacement.

The existing path captures each list's length before evaluating its index once.
Every completed index demands the enclosing reservations; final acquisition demands
all of them, then ends them. Cancellation skips unfinished bounds/acquisition while
preserving completed effects and earlier reservation demand. Reservations grant no
loan authority. Returning index mutation and overlapping accesses remain E302.
Reborrows preserve parent identity/suspension, and dynamic elements still overlap.

Shared sibling headers, old copies, calls, last use, owner reset/completion and
Leave retain their prior behavior. Indirect stores/calls still invalidate Boolean
knowledge used for carried initialization. Indexed SetPath writes remain gated
before reservations in `loans/control.rs`. Whole-list exclusive values, reference/
temporary roots, broader carried shapes and exclusive headers remain separate.
See [the proof](EXCLUSIVE_RESTARTS.md#carried-list-elements) and
[the example](examples/exclusive-carried-elements.mwy).

## Actual validation

- The initial element-acceptance regression reproduced B001 at the collection gate.
- Focused `cargo test --locked --manifest-path compiler/Cargo.toml --target
  x86_64-unknown-linux-gnu --target-dir compiler/target exclusive_carried_elements`
  passed eight source groups and six graph groups. Eight native groups pass in
  debug and release, including the separately rerun panic-index case.
- Graph evidence covers exact Slot/Field/Element sources, inherited mutability,
  empty reservation authority, reservation expiry, parent identity and required
  Acquire before capture and at acquisition. Missing initialization, inactive/
  completed owners and malformed/immutable sources reject. Cancellation still
  requires initialization before capture, even when no loan is created.
- Native cases cover integer/Boolean/float layouts, nested fields, copies, calls,
  moves, children, disjoint shared headers, owner resets, once-only index effects,
  current initialized lengths, precise bounds spans, signed/unsigned/empty lists,
  Leave/panic cancellation, unfinished stores and primary rejection boundaries.
- Obsolete indexed-borrow rejection fixtures were narrowed to whole-list borrows;
  indexed-write gates remain tested.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 638 library and 590 native Rust tests (1228 total), 16 tooling
  plus 4 compiler-harness Python tests, and 78 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities do not count as language rejections or full release qualification.
- Local links, 23 catalog records and 7 schemas/6 examples passed. Whitespace checks
  passed; external links were not fetched. Gate log:
  `/tmp/meowy-exclusive-carried-elements-gate.log`.
- No backend, runtime, editor, reference fixture or dependency changes were needed;
  editor and separate runtime/sanitizer gates were not rerun.

## Prior capabilities and other areas

Carried reference-free lists retain whole initialized length/payload through inner
restarts. Shape limits remain 256 parts/32 levels; list construction retains capacity
65,536, layout 1 MiB and one-based initialized-length checks. Nullable/union/reference-
bearing/foundation/owning and top-level unit carried slots retain their gates.

Shared whole-list views, nested element/record-field projections and reborrows
retain canonical Slot/Field/Element sources and parent reference identity. Index
expressions execute once and check current initialized length. Views can survive
inner restarts and alias scope exit while their result owner lives; owner expiry
cannot be undone by reinitializing the same physical site.

Selected-slot mutability is independent of whole-binding replacement. `Proofs.mutable`
tracks replaceable roots and `Proofs.fields` mutable owned descendants; `variable`
drives snapshots/refinements without granting writes through shared references.
Pointer syntax uses tight `&`/`&!`/`*`, immediate-field `.&`/`.&!`/`.*` and grouping
for complete targets.

Standalone documentation supports attachment, checked links/signatures, E801-E805,
`doc check`/`doc build`, local API pages and checked/opt-in examples. Net/HTTP/TLS
remain specified library work; module/type/I/O/task foundations and capability-typed
lifecycle implementation precede executable adapters.

## Architecture and proof boundaries

| Responsibility | Existing owner |
| --- | --- |
| Carried shape and indexed-write gate | `src/borrow/carried.rs` |
| Whole-slot initialized/active state and Acquire | `src/loans/emission_init.rs` |
| Direct/projected acquisition and parent identity | `src/loans/transitive.rs` |
| Element evaluation and initialized-length checks | `src/list.rs` |
| Indexed acquisition/reservations and gated indexed writes | `src/loans/elements.rs`, `src/loans/control.rs` |
| Scalar source qualification and exclusive reset frontier | `src/loans/exclusive_restarts.rs` |
| Shared-header certificates | `src/loans/restart_headers.rs` |

## Still outside this compiler

The full module/package graph, generic specialization, captures, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Qualify indexed writes in `loans/control.rs` separately. Require active whole-slot
   initialization before owner capture and completed stores; preserve reservation
   ordering, captured index addresses, bounds, canceled RHS effects, owner expiry
   and final use. Add source/graph/native proof before removing `carried::storage`.
2. Preserve shared-header certificates, genuine call/input opacity and old-copy loans.
   Exclusive header carriage, broader carried shapes and owning cleanup remain
   separate. Continue module graphs/library foundations. Keep STATUS current, commit
   cohesive validated work, and do not push or recreate STEP logs.
