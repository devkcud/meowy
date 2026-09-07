# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Extended exclusive scalar list elements to mutable record fields and exact-backed
  emitted storage (`bbd08b3`; contract/example `8010888`). Canonical Place roots
  preserve named-field paths and target
  lifetimes; every mutable boundary and the complete backing type are checked.
- Capturing and reserving the selected list permits independent sibling access.
  Index effects run once; returning acquisition creates the root element loan.
  Cancellation preserves effects without retaining a future reservation demand.
- All ten compiler checks pass: 748 Rust tests (351 library, 397 native), 20 Python
  tests, 46 debug/release examples, formatting, Clippy, build, schemas/catalog and
  931 local links. Fourteen new native groups and three new graph groups pass.
- Nested-index/reference/temporary roots, wider elements, whole-list exclusive values
  and exclusive restart bodies remain gated. Runtime ABI, dependencies and reference
  fixtures did not change; runtime/editor and release qualification were not rerun.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Local, projected and emitted scalar list elements | Nested-index owners and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Define nested-index owned paths and per-collection reservations before extending
   exclusive element borrowing. Reuse bounded paths, preserve single evaluation and
   cancellation at every index, and validate mutable fields, exact alias backing and
   target lifetime. Keep reference/temporary roots and wider ownership shapes gated.
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
