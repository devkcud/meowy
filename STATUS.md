# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented owned exclusive scalar list-element borrowing for ordinary mutable
  local lists (`28ca2b0`; contract/example `cf4eca8`). A distinct HIR operation captures owner/length before one index
  evaluation; a no-authority reservation protects returning acquisition. Mutable-
  owner proof grants the final root element loan independently of shared references.
- Existing Element regions conservatively overlap same-list elements and owner
  access. Moves, reborrows, calls/results, scopes and cancelled indices preserve
  current authority and lifetimes. Bounds reuse E101/P001 and initialized length.
- All ten compiler checks pass: 731 Rust tests (348 library, 383 native), 20 Python
  tests, 45 debug/release examples, formatting, Clippy, build, schemas/catalog and
  928 local links. Fifteen native groups and three graph proof groups pass.
- Alias/field/indexed/reference/temporary roots, wider elements, whole-list exclusive
  values and exclusive restart bodies remain gated. No runtime ABI, dependency or
  reference fixture changed. Conformance remains 10 passed/13 unsupported/0 failed;
  runtime/editor suites were not rerun.
- Standard library/module loading, generated cleanup and complete v0.0.1
  qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Owned scalar list elements plus prior scalar/field storage | Projected/emitted list owners and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend owned list borrowing to mutable record-field and exact-backed emitted
   storage using canonical Place roots. Validate every mutable boundary and backing
   type, capture the selected list address/length once, and reserve that list's whole
   region through returning index evaluation. Keep reference-derived, temporary and
   nested-index roots gated until their separate authority/lifetime proofs exist.
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
