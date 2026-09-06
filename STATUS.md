# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `2eb9d1f` extends initialized assignment through nested lists, such as
  `matrix[row][column] = value`. SetElement carries an ordered IndexStep path;
  each layer reads its own initialized length and checks its index before the next.
  The final store changes only the selected leaf, preserving copies and other rows.
- P001 identifies the failing target prefix. A failed outer check skips later
  indices and RHS; panic/leave/restart paths stop at their actual last phase.
  Captured indices cannot be retargeted by later index-variable assignments.
- Parent reservations remain live at every returning bounds/address phase and the
  final store. Last-use shared reads are allowed; later root/row/element views and
  intervening returning writes fail E302. All root-list indices still overlap.
- Coverage/example: `06bdee8` adds seven native groups and
  `compiler/examples/nested-writes.mwy`. Three backend groups plus frontend/loan
  evidence cover nested layouts, unequal lengths, prefix spans and bounded paths.
- All 14 combined checks pass: 250 Rust tests, 35 Python tests, 855 local
  links, editors, schemas/catalog, formatting, Clippy, build and conformance.
  Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- The optimized compiler runs the nested-writes example with exact output.
  Independent review passed 11 directed cases without finding a blocker. Runtime
  debug/release/sanitizer checks pass unchanged; generated cleanup is still pending.
- No active implementation workers, unfinished code or failing checks remain.
  Mutable fields, writes through references/temporaries, exclusive references,
  slices, owned/reference elements, cancellation and DWARF remain future work.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Checked element borrows and nested local element writes | Contextual constraints, mutable fields, exclusive references and moves/cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend remaining contextual list constraints in `compiler/src/list_context.rs`.
   Start with effectful blocks whose result shape can be proved without executing
   or lowering effects during candidate selection. Preserve ambiguity and budget
   diagnostics; verify once-only effects, early exits and candidate widths.
2. Define mutable field shapes and exclusive-reference contracts before permitting
   field/reference write targets. Keep initialized Copy writes distinct from the
   move/drop state required by owned elements, slices and removal.
3. Define generated payload/diagnostic layouts and connect cleanup to runtime
   mark/close while parent storage lives. Retain owning outcomes, drain every failure
   batch and preserve interleaved cleanup before cancellation and unwinding.
4. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
5. Build the manifest/module graph for Meowy libraries and documented projects.
   Keep runtime, editor and library work visible here.
6. Run `python3 -B tools/verify.py --all` after integrations. LSan needs process
   inspection. Strict conformance still has 13 unsupported cases; the bootstrap
   gate and this host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
