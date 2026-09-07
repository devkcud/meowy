# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Added exclusive scalar field borrows beneath owned indexes, including nested and
  emitted storage (`1f8295c`; contract/example `397b3b2`; refactor `16e0230`).
  Complete paths preserve canonical leaf regions, exact backing,
  mutable boundaries, target lifetime and enclosing collection reservations.
- Index effects and initialized-length checks run once in order. Field acquisition
  adds no synthetic index; cancellation, moves, child/call transfer and captured
  stores preserve the existing scalar ownership behavior.
- All ten compiler checks pass: 785 Rust tests (357 library, 428 native), 20 Python,
  48 debug/release examples, formatting, Clippy, build, schemas/catalog and 936 links.
  Sixteen new native groups and three graph groups pass. The representation refactor
  separately passed the full gate before the new behavior was enabled.
- Reference/temporary roots, wider pointees, exclusive-bearing carriers, whole-list
  exclusive values and exclusive restart bodies remain gated. Runtime ABI, dependencies
  and reference fixtures did not change. Runtime/editor and release qualification
  were not rerun; conformance still has 13 unsupported cases.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scalar elements and field leaves through owned indexed paths | Generated cleanup and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Define generated payload/diagnostic layouts and scope-cleanup integration. Map
   backend exits to runtime cleanup Stack::mark/unwind and Task::mark/close, retaining
   parent storage and owning outcomes, draining failures and preserving interleaved
   cleanup before cancellation/unwinding. Prove each integration with runtime/native
   checks; current loan events do not implement generated destruction.
2. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
3. Extend aggregate/emitted-name/cross-element constraints in the list-context
   modules without replay or stale facts. Build the manifest/module graph for Meowy
   libraries and documented projects; keep runtime/editor/library progress visible.
4. Preserve current compiler evidence; do not rerun green checks without a new change
   or concern. Run `python3 -B tools/verify.py --all` after wider integrations. LSan
   needs process inspection. Strict conformance still has 13 unsupported cases;
   this bootstrap gate and host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
