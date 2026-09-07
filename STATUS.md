# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented bounded access records in `698e4b1` on the existing loan CFG: direct reads,
  stored-tag inspections, shared acquisitions and writes. Direct targets retain
  canonical storage, lexical views and typed paths; pointee targets retain exact
  immutable pointer value IDs. Public bounds are not converted into physical reads.
- Existing write conflicts and last-use behavior are preserved. Tag-only paths do
  not consume payloads; static type predicates create no invented tag reads.
  Branches keep their guards and header/reset transfers add no accesses. Missing
  pointee evidence is allowed only on unreachable paths; reachable gaps remain B001.
- All ten compiler checks pass: 562 Rust tests (304 library, 258 native), 20 Python
  tests, 879 local links, schemas/catalog, formatting, Clippy, build and conformance.
  All 37 examples execute in debug/release. Twelve new graph groups prove ordering,
  paths, aliases, pointer versions, guards, resets, missing evidence and budgets.
- The [exclusive-reference design](compiler/EXCLUSIVE_REFERENCES.md) remains the
  continuation. Access records are the implemented foundation; stable authority,
  parent suspension and forward availability are still required. `&!` and indirect
  writes remain B001. Backend code, runtime ABI and dependencies did not change.
- Terminal expiry remains implemented in `7906333`, with native/example evidence
  in `324e9ae`. Existing replay/work/storage limits remain; access metadata counts
  toward the current graph-origin/work limits. Complete source changes and checks
  are recorded in the adjacent logs, including initial failures and corrections.
- Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
  Runtime/editor suites were not rerun; their historical evidence at `f16c30b`
  remains in the compiler handoff. Complete v0.0.1 qualification is still open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Bounded access records and terminal restart expiry | Guarded authority and forward initialization on the current CFG |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add guarded authority alternatives and parent relationships in
   `compiler/src/borrow_value.rs` and `compiler/src/loans/`, distinct from physical
   sources, public bounds and immutable value IDs. Resolve direct/pointee access
   regions through that authority, then add forward initialized/moved state on the
   same CFG. Prove reinitialization, child copies, parent suspension, guarded joins
   and exact Leave behavior before enabling the designed scalar `&!` slice. Keep
   inferred carriers, derived shared call/cell crossings and exclusive restarts gated.
2. Define generated payload/diagnostic layouts and connect cleanup to runtime
   mark/close while parent storage lives. Retain owning outcomes, drain every failure
   batch and preserve interleaved cleanup before cancellation and unwinding.
3. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
4. Extend aggregate/emitted-name/cross-element constraints in the list-context
   modules without replay or stale facts. Build the manifest/module graph for Meowy
   libraries and documented projects; keep runtime/editor/library progress visible.
5. Preserve current compiler evidence; do not rerun green checks without a new change
   or concern. Run `python3 -B tools/verify.py --all` after wider integrations. LSan
   needs process inspection. Strict conformance still has 13 unsupported cases;
   this bootstrap gate and host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
