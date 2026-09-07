# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Extended exclusive scalar list-element borrows through nested indexes and mutable
  fields, including emitted storage (`d4cd292`; contract/example `345cf64`).
  Owned paths preserve actual layout, canonical
  source identity, exact backing and target lifetime.
- Indexes run once in order, with each list checked before the next index. Enclosing
  reservations carry no authority and remain demanded through final acquisition.
  Later cancellation retains protection needed by earlier completed bounds checks;
  effects after a non-returning index do not execute.
- All ten compiler checks pass: 766 Rust tests (354 library, 412 native), 20 Python
  tests, 47 debug/release examples, formatting, Clippy, build, schemas/catalog and
  933 local links. Fifteen new native groups and three new graph groups pass.
- Scalar field leaves after indexes, reference/temporary roots, wider pointees,
  whole-list exclusive values and exclusive restart bodies remain gated. Runtime ABI,
  dependencies and reference fixtures did not change. Runtime/editor and release
  qualification were not rerun; conformance still has 13 unsupported cases.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Local, projected, emitted and nested scalar list elements | Scalar field leaves after indexes and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend owned indexed paths to scalar field leaves such as `&!rows[i].value`.
   Preserve mutable boundaries, exact alias backing and enclosing reservations through
   field acquisition. Verify mixed paths, cancellation, overlap and target lifetime;
   keep reference/temporary roots and non-scalar pointees separately gated.
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
