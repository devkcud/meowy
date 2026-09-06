# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `a978c8b` implements checked initialized-element assignment to direct
  mutable local Copy lists: `values[index] = value`. It captures the original
  initialized length, evaluates/checks the index before the RHS, then stores only
  the selected element. Length, other elements and earlier list copies survive.
- Internal parent-storage reservations reject owner/element writes during returning
  index/RHS evaluation. Shared reads may finish before the store; any later shared
  use conflicts with the final exclusive access (E302). All indices conservatively
  overlap. No source-level exclusive reference or owned-element support is implied.
- Bounds remain E101/P001 with the target byte span. Bounds failure skips RHS;
  leave/restart/panic operands skip the remaining assignment. Dead paths do not
  invent accesses. Direct root writes preserve existing Copy-read snapshots.
- Coverage/example: `2a15a37` adds seven native groups and
  `compiler/examples/element-writes.mwy`. Library coverage adds three backend,
  one list and one loan group. E305, E207 and B001 boundaries remain explicit.
- All 14 combined checks pass: 238 Rust tests, 35 Python tests, 854 local
  links, editors, schemas/catalog, formatting, Clippy, build and conformance.
  Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- The optimized compiler runs the element-writes example with exact output.
  Independent ownership review found no remaining blocker. Runtime debug/release/
  sanitizer checks pass unchanged; generated task cleanup remains separate work.
- No active implementation workers, unfinished code or failing checks remain.
  Nested write targets, field/reference/temporary targets, exclusive references,
  slices, owned/reference elements, cancellation and DWARF remain future work.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Checked element borrows and direct local element writes | Nested write paths, exclusive references, moves/cleanup and contextual constraints |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend checked write paths to nested list targets in the HIR, checker, loans and
   backend. Evaluate each index/bounds check once from root to leaf, retain parent
   reservations, and test aliases plus early exits. Immutable field/reference owners
   stay excluded until their mutability/exclusive contracts are implemented.
2. Extend remaining effectful/non-scalar contextual constraints without replaying
   effects or weakening budgets. Keep annotations/B001 for unproved candidates.
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
