# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented scalar exclusive references (`3fe8715`, example/docs `87e7926`)
  to ordinary mutable boolean/integer/float
  locals: moves, reinitialization, shared/exclusive reborrows and indirect stores.
  Guarded availability reports E301/E309; lifetime and mutability remain E303/E305.
- Mode-aware LoanIds and bounded ancestor walks enforce parent suspension and E302
  conflicts. Stores capture their pointer before RHS work and finish only on returning
  paths. Conditional moves, short circuits, named Leave and panic are covered.
- Exclusive signatures, carriers, cells, aliases, fields/collections, comparisons,
  block results, dispatch and restart bodies remain B001. Shared values retaining
  exclusive ancestry cannot cross call/result/cell/dispatch boundaries.
- All ten compiler checks pass: 609 Rust tests (334 library, 275 native), 20 Python
  tests, 38 examples in debug/release, formatting, Clippy, 881 local links, schemas,
  catalog, build and conformance. Seventeen new native groups cover the exclusive
  slice. Reference fixtures and runtime ABI remain unchanged.
- Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
  Runtime/editor suites were not rerun; their historical evidence remains in the
  compiler handoff. Complete v0.0.1 qualification remains open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scalar exclusive references, moves and guarded permissions | Exclusive function contracts and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Preserve the first scalar exclusive slice and its native matrix. Design explicit
   exclusive function contracts in `compiler/src/borrow_contract/` and
   `compiler/src/loans/permissions.rs`: prove symbolic input overlap, guarded authority
   transfer and conservative public bounds before opening those gates. Extend other
   excluded shapes one at a time; keep unsupported source forms explicit.
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
