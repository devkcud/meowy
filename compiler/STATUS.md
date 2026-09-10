# Compiler handoff and work tracker

Updated: 2026-09-10. Indexed writes to carried lists are complete and passed the
compiler gate. No failing checks remain.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the project;
[../COMPILER.md](../COMPILER.md) records the plan. Keep this handoff current;
Git holds history. Do not recreate STEP logs.

## Current compiler slice

Indexed SetPath in `loans/control.rs` now checks containing-slot Acquire before
owner capture and again at the completed store. The obsolete `carried::storage`
gate is removed; bounded shape validation remains. The active owner and entire
initialized slot are required even when an index or RHS later cancels. Static
field-only assignments keep their previous behavior.

The existing reservation names the first list region, including its leading
field path. Every returning index/bounds phase and final store demands it; no loan
authority is created. Returning index/RHS replacement or nested writes in that
region remain E302, while disjoint holder siblings remain writable. Different
elements and their fields still overlap conservatively within the first list.

Indices execute once, with each initialized length captured before its index and
bounds checked before later effects. The selected address survives RHS changes to
index variables. Scalar/record/list/string/unit leaves retain ordinary contextual
typing and selected-slot mutability. Surrounding lengths, siblings and old copies
are preserved. Final-use shared RHS reads may end before the write.

Leave/Restart/panic cancels unfinished stores and preserves completed effects and
earlier reservation demand. Owner reset cancels a pending index/RHS store and clears
initialization; the fresh owner needs a new emission. Owner expiry, reference copies,
shared-header certificates and local exclusive sibling frontiers retain their rules.
Whole-list exclusive values, reference/temporary-derived writes and broader carried
shapes remain gated. See [the proof](OWNERSHIP.md#carried-indexed-writes) and
[the example](examples/carried-writes.mwy).

## Actual validation

- The first indexed-write regression reproduced B001 at the removed collection gate.
- Focused `cargo test --locked --manifest-path compiler/Cargo.toml --target
  x86_64-unknown-linux-gnu --target-dir compiler/target carried_writes` passed nine
  source groups and four graph groups. Eight native groups pass in debug/release,
  including the separately rerun interrupted-owner-reset case.
- Graph evidence covers canonical first-list reservation/store identity, no loan
  authority, capture/final-store Acquire, and reservation demand at each returning
  phase. Missing Emit, inactive/completed owners and cancellation without initialized
  capture reject at the target span. Static field paths gain no index reservation.
- Native evidence covers scalar/aggregate layouts, copied values, unchanged lengths,
  contextual types, selected mutability, captured indices, final-use reads, mixed
  shared/exclusive headers, precise dynamic bounds spans, signed/unsigned/empty-list
  bounds, owner resets, Leave/Restart/panic cancellation and primary rejections.
- Obsolete indexed-write gates now test whole-list borrows/shared-reference writes
  or accepted writes after final exclusive use.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 651 library and 598 native Rust tests (1249 total), 16 tooling
  plus 4 compiler-harness Python tests, and 79 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities do not count as language rejections or full release qualification.
- Local links, 23 catalog records and 7 schemas/6 examples passed. Whitespace checks
  passed; external links were not fetched. Gate log: `/tmp/meowy-carried-writes-gate.log`.
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
| Bounded carried shape eligibility | `src/borrow/carried.rs` |
| Whole-slot initialized/active state and Acquire | `src/loans/emission_init.rs` |
| Direct/projected acquisition and parent identity | `src/loans/transitive.rs` |
| Element evaluation and initialized-length checks | `src/list.rs` |
| Indexed acquisition and write reservations | `src/loans/elements.rs`, `src/loans/control.rs` |
| Scalar source qualification and exclusive reset frontier | `src/loans/exclusive_restarts.rs` |
| Shared-header certificates | `src/loans/restart_headers.rs` |

## Still outside this compiler

The full module/package graph, generic specialization, captures, public FFI, wider
ownership/cleanup, executable networking, public artifacts/replay and LSP remain
separate. Host execution does not qualify minimum platforms or bundled distributions.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Begin the file-module foundation from `src/check/names.rs` import lookup,
   `src/lib.rs::compile` and `src/driver.rs` single-file loading/manifest refusal.
   Read `../docs/reference/modules-and-ffi.md` and
   `../docs/reference/packages-and-builds.md` first.
   Define a bounded relative-file import slice with canonical identity, exports,
   diagnostics with file identity, cycle rejection and once-only ordered initialization.
   Preserve foundational lookup and explicit gates for package/manifest features;
   add multi-file fixtures before enabling execution and run the compiler gate.
   Do not approximate imports by textual concatenation or silently ignore manifests.
2. Preserve shared-header certificates, call/input opacity, old-copy loans and
   owner expiry. Broader carried shapes, owning cleanup and exclusive header carriage
   remain separate. Keep STATUS current, commit cohesive validated work, and do not
   push or recreate STEP logs.
