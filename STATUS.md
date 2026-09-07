# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented scalar-field exclusive borrows through mutable emitted reference-free
  Copy records (`12bee5a`; docs/example `69353bd`). Existing field mutability and exact whole-record backing checks now
  govern alias projections; even widening an unselected field remains B001.
- Canonical Slot roots, nested field indexes and lexical views preserve target-owned
  lifetime and guarded identity. Sibling/primary access, ancestor conflicts, moves,
  calls/blocks, captured stores and cancelled fallback layouts reuse existing passes.
- All ten compiler checks pass: 713 Rust tests (345 library, 368 native), 20 Python
  tests, 44 debug/release examples, formatting, Clippy, build, schemas/catalog and
  921 local links. Fourteen new native groups and canonical/backing evidence pass.
- Whole-record exclusive pointees, reference cells, indexed/union/reference paths,
  owning carriers and exclusive restart bodies remain gated. No backend, ABI,
  dependency or reference fixture changed. Conformance remains 10 passed/13
  unsupported/0 failed; runtime/editor suites were not rerun.
- Standard library/module loading, generated cleanup and complete v0.0.1
  qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Exclusive scalar locals and fields across ordinary/emitted storage | Scalar list-element borrows and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Design exclusive scalar list-element borrows on ordinary mutable bounded lists.
   Trace `compiler/src/list.rs`, `compiler/src/loans/transitive.rs` and collection
   access regions. Prove owner authority, once-only index/bounds evaluation,
   conservative element overlap, live-owner conflicts and cancellation before opening
   `&!items[index]`. Do not infer exclusive permission from an internal shared view.
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
