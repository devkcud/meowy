# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Implemented anonymous scalar-reference block results (`65eda96`, docs/example
  `f3fa667`) using existing guarded loan identities and consuming emissions. Retained slots protect loans through completion;
  named Leave returns initialized results, while cancellation keeps moves/effects
  without inventing future result use. Local escapes remain E303.
- Explicit dispatch-block metadata keeps captured exclusive results gated even when
  the receiver is scalar. Named fields, carriers, cells and exclusive restart bodies
  retain their capability boundaries. Missing cancellation proof reports B001.
- The new short-circuit matrix exposed a separate Never-operand typing bug, fixed
  in `1ff86ca`. Unary
  and non-boolean binary operators now propagate non-returning operands while keeping
  operand checking and evaluation order. All six focused control groups pass.
- All ten compiler checks pass: 665 Rust tests (339 library, 326 native), 20 Python
  tests, 41 debug/release examples, formatting, Clippy, build, schemas/catalog and
  904 local links. No backend, ABI, dependency or reference fixture changed.
  Conformance remains 10 passed/13 unsupported/0 failed; runtime/editor suites
  were not rerun.
- Standard library/module loading, generated cleanup and complete v0.0.1
  qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scalar exclusive values, calls and anonymous block results | Scalar-field borrows and wider ownership shapes |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Design exclusive borrows of scalar fields in mutable reference-free Copy records.
   Reuse `compiler/src/check/references.rs`, existing write-path mutability rules and
   `compiler/src/loans/access.rs` canonical regions. Prove sibling disjointness,
   whole-owner conflicts, lifetime, moves and call/block transfer before admitting
   `&!owner.field`. Keep reference cells, owned carriers and generated cleanup separate.
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
