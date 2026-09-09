# Compiler handoff and work tracker

Updated: 2026-09-09. Carried reference-free record initialization is implemented
and passed the compiler gate. No failing compiler checks remain. Full v0.0.1 is
incomplete. [../STATUS.md](../STATUS.md) tracks the whole project;
[../COMPILER.md](../COMPILER.md#documentation-completion-slice) records the plan.
Keep this handoff current; Git holds history. Do not recreate STEP logs.

## Current compiler slice

Declared reference-free record result slots now use the existing whole-slot
initialization proof across inner restarts. `borrow/carried.rs::eligible` accepts
nested scalar/unit records within 256 type parts and 32 levels, charging shared
proof work; `check/statements.rs` applies the same check before deferring a slot.
The existing CFG proof still requires exactly-once initialization on every owner
completion, preserves ancestor slots across inner restarts and clears owner slots
on reset. No per-field initialization, fabricated reads or backend changes were
introduced.

Copies, mutable field writes, whole-record replacement, scalar sibling exclusive
loans, owner resets, Leave and partial panics have debug/release coverage. Borrowing
original carried record storage or its fields remains B001 in `carried::validate`;
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

- Focused carried-record source/shape coverage: nine tests passed using
  `cargo test --locked --manifest-path compiler/Cargo.toml --target x86_64-unknown-linux-gnu --target-dir compiler/target --lib carried_record`.
- `python3 -B tools/verify.py --compiler`: all 10 selected checks passed, including
  fmt, Clippy, build, repository contracts, 569 library and 541 native Rust tests
  (1110 total), 20 Python tests and 70 examples executed in debug and release.
- Five new native test groups exercise retained payloads, old copies and writes,
  owner reset/Leave, partial initializer panic and rejected initialization/borrows.
- Conformance: 10 passed, 13 unsupported, 0 failed in both profiles. Unsupported
  capabilities remain outside the full language gate.
- Final documentation check: 1053 local links in 101 Markdown files, 0 failures;
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
   and carried plain-record initialization slice are complete; do not reopen them
   as preparation for further documentation polish.
2. Qualify shared borrows of carried record storage and nested field projections
   through `borrow/carried.rs`, `loans/emission_init.rs` and the existing source/
   physical-storage solver. Require active and fully initialized storage on every
   acquisition path, preserve owner expiry/reset and old-copy lifetimes, and avoid
   synthetic payload reads. Keep exclusive projected loans gated pending their
   separate path/frontier proof; keep nullable/union/reference-bearing shapes out.
3. Add focused acceptance, stale/early acquisition, owner reset, alias lifetime,
   field projection and debug/release execution cases before removing the storage
   borrow gate. Run focused Rust tests and `tools/verify.py --compiler`.
4. Continue module graphs and library foundations independently of optional doc
   polish. Broader carried lists, tags, owned cleanup and richer value-state proofs
   need their own bounded implementation and evidence.
5. Keep root/compiler STATUS current after logical steps, commit cohesive validated
   work and do not push or recreate STEP logs.
