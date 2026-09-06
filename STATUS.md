# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Organization is complete for the remaining large ownership/parser/native-suite
  modules: native `c83f1f1`, parser `d599149`, borrow `f550947`,
  loans `717f5af`. Entry files are now 2, 171, 83 and 49 lines respectively,
  with focused implementation/test files under their owning directories.
- Native execution remains one Cargo target with one Case/NEXT/Drop harness and
  all 126 test functions. All 21 example includes plus the conformance fixture
  resolve to the same file contents. Tests are grouped into 19 behavior modules.
- Parser grammar, root entrypoints, borrow facts/reexports, origin transfers,
  loan graph construction/solving, budgets and diagnostics are preserved. Function
  and literal audits accompany focused before/after tests; no language feature
  or reference fixture changed in this pass.
- All 14 combined checks pass: 266 Rust tests, 35 Python tests, 857 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs dynamic-lists with exact output. The prior primitive
  suffix inference remains `932297a`, with coverage/example `89b530c`; earlier
  backend/checker/list-context organization remains in their focused modules.
- Both AGENTS files retain advisory organization guidance without a size gate.
  No active workers, unfinished code or failing checks remain. Next implementation
  work is mutable field metadata and checked access, followed by broader ownership,
  generated cleanup, modules/library support, cancellation and release qualification.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Modular parser/ownership/backend and runtime-valued suffix inference | Mutable record fields, checked access and broader ownership |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Preserve record-field mutability from `compiler/src/ast.rs` through
   `compiler/src/hir.rs` and `compiler/src/check/{names,blocks,statements}.rs`.
   Mutability belongs to the
   record shape; verify expected types, construction and E206 branch consistency
   before enabling writes. Keep representation changes in the owning modules.
2. Add checked field writes only after metadata survives the full type pipeline.
   Require a mutable field and exclusive owner access; verify RHS order, static
   overlap and last-use loans. Keep owned moves/cleanup and source-level exclusive
   references separate until their initialization/lifetime rules are implemented.
3. Define generated payload/diagnostic layouts and connect cleanup to runtime
   mark/close while parent storage lives. Retain owning outcomes, drain every failure
   batch and preserve interleaved cleanup before cancellation and unwinding.
4. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
5. Extend aggregate/emitted-name/cross-element constraints in the new list-context
   modules without replay or stale facts. Build the manifest/module graph for Meowy
   libraries and documented projects; keep runtime/editor/library progress visible.
6. Run `python3 -B tools/verify.py --all` after integrations. LSan needs process
   inspection. Strict conformance still has 13 unsupported cases; the bootstrap
   gate and this host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
