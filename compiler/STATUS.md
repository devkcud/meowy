# Compiler handoff and work tracker

Updated: 2026-09-10. Local exclusive scalar fields in list-containing carried
records are complete and passed the compiler gate. No failing checks remain.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the project;
[../COMPILER.md](../COMPILER.md) records the plan. Keep this handoff current;
Git holds history. Do not recreate STEP logs.

## Current compiler slice

Direct acquisition in `loans/transitive.rs::referenced` now reaches the existing
exclusive restart source/frontier proof without a blanket list-containing-slot
rejection. `exclusive_restart_source` already requires canonical owner/root/view,
exact named record-field paths, mutable Boolean/integer/float leaves and no live
exclusive loan or descendant across reset. It needs no new representation or rule.

Every direct root retains containing-slot Acquire: the owner must be active and
its entire slot initialized, including list siblings. Physical paths preserve
field disjointness. A list sibling may be replaced while a scalar-field loan lives;
whole-record access, overlapping loans and parent use under live children remain
E302. Mutable fields remain writable under immutable owned aliases/parent fields.
Shared-reference access does not grant mutation permission.

Old value copies remain independent. Moves, calls, reborrows, final use, owner
reset, completion and Leave retain their existing rules. Certified shared list
headers may coexist with local exclusive scalar loans. Genuine call/input opacity
and exclusive descendants still reject reset frontiers. Indirect stores/calls
still invalidate Boolean knowledge, so loop flags may need restoring afterward.

`carried::storage` remains at ExclusivePath and indexed SetPath before reservations.
Exclusive list pointees and indexed paths remain B001; wider exclusive pointees,
exclusive headers and unsupported carried shapes remain separate work. See
[the proof](EXCLUSIVE_RESTARTS.md#carried-record-fields) and
[the example](examples/exclusive-carried-list-fields.mwy).

## Actual validation

- The first field-acceptance regression reproduced B001 from the blanket list gate.
- Focused `cargo test --locked --manifest-path compiler/Cargo.toml --target
  x86_64-unknown-linux-gnu --target-dir compiler/target exclusive_carried_record`
  passed 12 source/graph groups and eight native groups. Native cases run in both
  debug and release. Existing plain-record cases also run with list siblings.
- Source/native coverage includes nested integer/Boolean/float fields, copies,
  moves, shared/exclusive children, parent suspension, disjoint list replacement,
  shared headers, calls, owner resets, Leave, expiry, final use and primary gates.
- Graph evidence preserves exact Slot/Field identity and whole-slot Acquire.
  Missing Emit, inactive owner and acquisition after Complete fail at the borrow
  span. Corrupted owner/root/view, invalid fields and indexed/non-scalar paths fail.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 624 library and 582 native Rust tests (1206 total), 16 tooling
  plus 4 compiler-harness Python tests, and 77 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities do not count as language rejections or full release qualification.
- Local links, 23 catalog records and 7 schemas/6 examples passed. Whitespace checks
  passed; external links were not fetched. Gate log:
  `/tmp/meowy-exclusive-list-fields-gate.log`.
- No backend, runtime, editor, reference fixture or dependency changes were needed.
  Editor and separate runtime/sanitizer gates were not rerun.

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
| Carried shape and indexed collection gate | `src/borrow/carried.rs` |
| Whole-slot initialized/active state and Acquire | `src/loans/emission_init.rs` |
| Direct/projected acquisition and parent identity | `src/loans/transitive.rs` |
| Element evaluation and initialized-length checks | `src/list.rs` |
| Exclusive indexed acquisition and indexed writes | `src/loans/elements.rs`, `src/loans/control.rs` |
| Scalar source qualification and exclusive reset frontier | `src/loans/exclusive_restarts.rs` |
| Shared-header certificates | `src/loans/restart_headers.rs` |

## Still outside this compiler

The full module/package graph, generic specialization, captures, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Investigate exclusive scalar list elements in `loans/elements.rs` separately.
   Qualify containing-slot initialization, once-only index evaluation, reservation
   order, bounds, canonical element identity, parent authority and reset frontiers.
   Preserve indexed SetPath gates until that independent path is qualified. Add
   source/graph/native acceptance and boundary tests, then run the compiler gate.
2. Then investigate indexed writes in `loans/control.rs`, including canceled RHS,
   completed reservations, list sibling replacement, owner expiry and final use.
   Preserve shared-header certificates, genuine opacity and old-copy loans.
3. Continue module graphs/library foundations. Broader carried shapes, owning
   cleanup and exclusive header carriage remain separate. Keep STATUS current,
   commit cohesive validated work, and do not push or recreate STEP logs.
