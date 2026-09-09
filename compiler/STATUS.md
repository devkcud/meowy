# Compiler handoff and work tracker

Updated: 2026-09-09. Local exclusive carried-record scalar-field borrows are
implemented and passed the compiler gate. No failing compiler checks remain.
Full v0.0.1 is incomplete. [../STATUS.md](../STATUS.md) tracks the whole project;
[../COMPILER.md](../COMPILER.md) records the implementation plan.
Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Current compiler slice

`loans/exclusive_restarts.rs::exclusive_restart_source` now follows exact mutable
record fields from the containing carried slot to a Boolean/integer/float leaf.
Canonical owner/root/view and empty origin-component checks remain mandatory;
invalid indexes, element paths, intermediate scalars and non-scalar leaves fail
closed. Path traversal consumes the existing graph budget.
`borrow/carried.rs::validate` no longer applies a blanket exclusive-record gate;
it still validates bounded scalar/reference-free record carried shapes.

Existing Borrow HIR emits whole-slot Acquire for both shared and exclusive
acquisitions. The containing owner must be active and its entire slot initialized.
No per-field initialization, fabricated reads or backend changes were introduced.
The reset-frontier proof still rejects every live exclusive loan or descendant,
including shared children and call-returned views. Certified shared sibling headers
retain their independent ancestry; genuine call/input opacity remains conservative.

Nested paths, widths, disjoint siblings, old copies, moves, parent/child authority,
final use, calls, owner reset and Leave have source and native coverage. E301/E302/
E303 retain move, conflict and expiry semantics. Indirect stores and exclusive calls
still invalidate Boolean initialization knowledge. Whole-record, string/unit and
other non-scalar exclusive paths remain B001. Ordinary local exclusive roots in
reset graphs remain outside this slice.
See [the contract](EXCLUSIVE_RESTARTS.md#carried-record-fields) and
[the example](examples/exclusive-carried-records.mwy).

Declared reference-free records retain whole-slot initialization across inner
restarts within 256 type parts and 32 levels. Shared references and projected
reborrows survive inner restarts/alias scope exit while their result owner lives.
Owner completion/reset expires old sources. Nullable, union, list, reference-bearing,
foundation and top-level unit carried slots remain gated. Ordinary copies and
completed-result locals retain their existing borrowing rules.

## Actual validation

- The new nested-field test first reproduced B001 from the old blanket record gate.
- Focused `cargo test --locked --manifest-path compiler/Cargo.toml --target
  x86_64-unknown-linux-gnu --target-dir compiler/target exclusive_carried_record`
  passed 7 source groups, 3 graph groups and 6 native groups. Native cases execute
  in both debug and release. Obsolete scalar-field B001 fixtures were changed to
  whole-record rejection only after the accepted native cases passed.
- Graph evidence checks exact paths, target-owned storage, precise local authority,
  Acquire without synthetic reads, active/initialized acquisition, malformed
  root/view/target/component identities and non-scalar/indexed paths.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, 587 library and 552 native Rust tests (1139 total), 16 tooling
  plus 4 compiler harness Python tests, and 72 examples in debug and release.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities remain outside the full language gate.
- Final documentation check passed 1077 local links in 102 Markdown files;
  `git diff --check` passed. Repository contracts also cover 23 catalog records
  and 7 schemas/6 examples. External links were not fetched.
- Editor and separate runtime/sanitizer gates were not rerun. No runtime, backend,
  editor, reference fixture or dependency files changed.

## Other area handoff

Standalone documentation supports structural attachment, checked symbol links,
compiler-derived signatures, E801-E805 diagnostics, `doc check`, `doc build`, local
API pages and checked/opt-in examples. CLI and `meowy::compile` check complete
source/documentation input; the low-level AST checker does not invent metadata.
Package documentation, assets, public indexes and LSP/rename remain separate.

The net/HTTP specification merge is complete. `@"net"` owns TCP/UDP, concrete peer
capabilities and explicit startup/sender/receiver lifecycle; HTTP lives under
`net.http`. Follow the dependency-ordered networking plan in COMPILER.md. This
specification does not expand supported imports or runtime protocols.

## Architecture and proof boundaries

| Responsibility | Existing owner |
| --- | --- |
| Carried slot shape | `src/borrow/carried.rs`, `src/check/statements.rs` |
| Whole-slot initialized/active state | `src/loans/emission_init.rs` |
| Value/source lifetime and alias backing | `src/borrow/`, `src/borrow_value/` |
| Physical storage and loan authority | `src/loans/` |
| Shared restart-header coverage | `src/loans/restart_headers.rs` |
| Exclusive scalar paths and restart frontier | `src/loans/exclusive_restarts.rs` |

## Still outside this compiler

The complete module/package graph, generic specialization, captures, public FFI,
wider ownership and cleanup, executable net peers/HTTP adapters/TLS, full LSP,
public artifact/replay formats and release qualification remain separate work.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 are the recorded toolchain.

## Next steps

1. Investigate fixed-capacity reference-free carried list slots in
   `borrow/carried.rs`, `check/statements.rs` and `loans/emission_init.rs`.
   Establish whole-slot initialization, copies, replacement, owner resets and
   incomplete/duplicate rejection before expanding the eligibility gate.
   Keep indexed acquisition/reservation support separate until `loans/elements.rs`
   and existing path proofs qualify it with focused source/native coverage.
2. Preserve exact scalar field sources, active initialization, shared-header
   certificates, genuine call/input opacity, exclusive backedge rejection,
   owner expiry and old-copy loans. Run focused tests and `tools/verify.py --compiler`
   for the next capability. Nullable/union/reference-bearing/owning carried slots,
   whole-record exclusive values and exclusive header carriage remain separate.
3. Continue module graphs and library foundations independently of optional doc
   polish. Net needs capability types, transport/role checking and bounded
   startup/shutdown before peers, TLS or HTTP adapters become executable.
4. Keep root/compiler STATUS current after logical steps, commit cohesive validated
   work and do not push or recreate STEP logs.
