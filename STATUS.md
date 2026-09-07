# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented scalar exclusive arguments for direct functions with primitive
  results (`c7af472`; contract/example `d72d5d0`). Calls capture arguments once and validate shared/exclusive access at
  entry, including suspended parents and conflicting arguments that are otherwise
  unused. Passing a holder moves it; explicit or implicit reborrows preserve parents.
- Symbolic input loans retain mode, physical overlap and guarded parent permissions
  through nested and recursive calls. Later argument Leave/panic skips call entry
  while keeping completed moves. Mutating calls invalidate caller refinements.
- All ten compiler checks pass: 626 Rust tests (334 library, 292 native), 20 Python
  tests, 39 examples in debug/release, formatting, Clippy, build, schema/catalog checks
  and 888 local links. Seventeen new native groups cover the function-argument slice.
- Reference-return exclusive contracts, wider signatures, carriers, fields/cells,
  dispatch blocks and exclusive restart bodies remain B001. Scalar-local references
  remain in `3fe8715`, example/docs `87e7926`; reference fixtures and runtime ABI
  are unchanged. Full conformance remains 10 passed/13 unsupported/0 failed.
- Runtime/editor suites were not rerun. Standard library/module loading, generated
  cleanup and complete v0.0.1 qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scalar exclusive locals and direct function arguments | Guarded reference-result authority and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Design guarded reference-result authority in `compiler/src/borrow_contract/`
   and `compiler/src/loans/values.rs`. A returned view needs an explicit relationship
   to the captured input loan, including parent suspension and all-input lifetime
   bounds. Do not infer authority from matching addresses or public bounds. Prove
   identity/shared reborrow/guarded selection cases before opening return gates;
   preserve the scalar-local and function-argument matrices.
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
