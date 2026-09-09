# Compiler handoff and work tracker

Updated: 2026-09-09. Shared carried-record borrows are implemented
and passed the compiler gate. No failing compiler checks remain. Full v0.0.1 is
incomplete. [../STATUS.md](../STATUS.md) tracks the whole project;
[../COMPILER.md](../COMPILER.md#documentation-completion-slice) records the plan.
Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Current compiler slice

Shared borrows of carried record storage and projections passed the compiler
gate. `borrow/carried.rs` now gates only exclusive record-storage borrows.
`loans/transitive.rs::referenced` already attaches Acquire to direct and projected
slot sources; `loans/emission_init.rs` proves whole-slot active/initialized state.
Reborrows retain parent authority and physical source/lifetime checks. No solver
or backend rewrite is needed. All 17 focused record tests passed, including eight
new source/proof groups. Five new native groups and
`examples/carried-record-borrows.mwy` passed in debug and release. See the
[shared record contract](OWNERSHIP.md#shared-carried-record-borrows).

Declared reference-free record result slots now use the existing whole-slot
initialization proof across inner restarts. `borrow/carried.rs::eligible` accepts
nested scalar/unit records within 256 type parts and 32 levels, charging shared
proof work; `check/statements.rs` applies the same check before deferring a slot.
The existing CFG proof still requires exactly-once initialization on every owner
completion, preserves ancestor slots across inner restarts and clears owner slots
on reset. No per-field initialization, fabricated reads or backend changes were
introduced.

Copies, mutable field writes, whole-record replacement, scalar sibling exclusive
loans, owner resets, Leave and partial panics have debug/release coverage. Shared
record-storage borrows preserve exact projected paths and expire with their result
owner. Exclusive record-storage borrows remain B001 in `carried::validate`;
ordinary copies and completed-result locals retain their existing borrow rules.
Union, nullable, list, reference-bearing, foundation and top-level unit carried
slots remain gated. See [ownership](OWNERSHIP.md#carried-reference-free-records)
and [the example](examples/carried-records.mwy).

## Completed bounded documentation slice

Standalone source documentation supports structural attachment, checked symbol
links, compiler-derived signatures, E801-E805 diagnostics, `doc check`, `doc build`,
safe local API pages and checked/opt-in executable examples. Unclosed fences remain
E002. The CLI and `meowy::compile` check complete source/documentation input;
the low-level AST checker does not synthesize documentation metadata. The contract
is [the bootstrap profile](../docs/reference/documentation.md#implemented-bootstrap-profile).

Package documentation, assets, public index formats and LSP/rename are separate.
`pulldown-cmark` 0.13.4 is locked as compiler tooling only; generated programs do not
link the Rust compiler or its Markdown dependencies. Documentation CLI examples
passed in the preceding slice and were not manually rerun here.

## Actual validation

- Focused carried-record source/shape/borrow coverage: 17 tests passed using
  `cargo test --locked --manifest-path compiler/Cargo.toml --target x86_64-unknown-linux-gnu --target-dir compiler/target --lib carried_record`.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, repository contracts, 577 library and 546 native Rust tests
  (1123 total), 20 Python tests and 71 examples executed in debug and release.
- Eight new source/proof groups and five native groups exercise whole and projected
  shared borrows, early/inactive acquisition, retained aliases, old copies, field
  conflicts, disjoint writes, final use, owner reset/Leave and exclusive path gates.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities remain outside the full language gate.
- Final documentation check: 1057 local links in 101 Markdown files, 0 failures;
  `git diff --check` passed. External links were not fetched.
- Editor and separate native runtime/sanitizer gates were not rerun for this slice;
  no editor, runtime, backend or dependency files changed.

## Architecture and proof boundaries

| Responsibility | Existing owner |
| --- | --- |
| Carried slot shape and borrow boundary | `src/borrow/carried.rs`, `src/check/statements.rs` |
| Whole-slot initialized/active state | `src/loans/emission_init.rs` |
| Value/source lifetime and alias backing | `src/borrow/`, `src/borrow_value/` |
| Physical storage and loan authority | `src/loans/` |
| Shared restart-header coverage | `src/loans/restart_headers.rs` |
| Exclusive restart frontier | `src/loans/exclusive_restarts.rs` |
| Documentation integration | Standalone CLI and `meowy::compile`; reference profile above |

Local exclusive scalar loans must end before reset edges. Shared headers require
complete entry/backedge coverage before header-only opacity can be excluded from
conservative ancestry. Genuine call/input uncertainty and exclusive descendants
remain gated. Late-publication certificates, result/discarded alias backing,
source expiry and old-copy loans remain independent requirements.

## Still outside this compiler

The complete module/package graph, generic specialization, captures, public FFI,
wider ownership and cleanup, executable HTTP/TLS libraries, full LSP, public
artifact/replay formats and release qualification remain separate implementation
work. Do not equate a green bootstrap gate with the documented v0.0.1 language.
Rust 1.98.1 and LLVM/Clang/LLD/LLVM ar 22.1.8 remain the recorded toolchain.

## Resume here

1. Read root/compiler rules and this handoff. The standalone documentation slice
   and carried plain-record/shared-borrow slices are complete; do not reopen them
   as preparation for further documentation polish.
2. Qualify local exclusive borrows of scalar fields in carried records through
   `borrow/carried.rs`, `loans/exclusive_restarts.rs` and the existing place/path
   proof. Require active, initialized containing storage and exact projected source
   identity. The exclusive loan and its descendants must end before restart edges.
   Keep whole-record/non-scalar exclusive paths and nullable/union/reference-bearing
   carried shapes gated; preserve shared header coverage and genuine opaque ancestry.
3. Add focused source/native coverage for nested scalar paths, sibling conflicts,
   shared descendants, final use, owner reset and rejected live backedge loans.
   Run focused Rust tests and `tools/verify.py --compiler` before removing the
   corresponding exclusive path restriction.
4. Continue module graphs and library foundations independently of optional doc
   polish. Broader carried lists, tags, owned cleanup and richer value-state proofs
   need their own bounded implementation and evidence.
5. Keep root/compiler STATUS current after logical steps, commit cohesive validated
   work and do not push or recreate STEP logs.
