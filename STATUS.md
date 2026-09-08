# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Owning-HIR design](compiler/OWNING_HIR.md) selects `strings.Owned` and defines
  initialized state, retained emissions and cleanup schedules. Source enablement
  first needs owning panic propagation, dynamic string-view origins and foundational
  types. Repeated ancestor writes need bounded cleanup-entry reclamation.
  Repository contract checks pass: 16 tooling tests, 957 links, catalog and schemas.
  No compiler behavior changed or execution suites rerun.

- Added private generated payload descriptors, explicit Owned initialization and
  atomic relocation across cleanup frames (`4df0e44`; LLVM proof `6d2d2b0`;
  contract `161543e`). Destination compatibility and cleanup
  slots are preflighted; failure retains the source and successful transfer arms the
  destination before disarming the old obligation.
- Generated callbacks preserve self-pointer relocation, exactly-once release and the
  enclosing panic cause. Static descriptor and caller-owned storage lifetimes remain
  explicit. Normal Meowy code generation still has no automatic owning-value cleanup.
- Prior combined validation passed all fourteen checks: 789 Rust (361 library, 428 native), 35 Python,
  48 debug/release examples, Vim/Neovim, 940 links, formatting, Clippy, build and
  schema/catalog checks. Runtime passes 92 case groups per debug/release/sanitized
  profile plus required fatal/guard/admission/fiber probes.
- Seven new native ownership groups and two LLVM groups pass. A generated relocation
  ELF imports only libc.so.6. Conformance still has 13 unsupported cases; private ABI
  proof does not qualify automatic cancellation, DWARF, minimum hosts or release.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scalar ownership, generated bridge and owning-HIR design | Owning panic propagation, then bounded resource cleanup |
| Runtime | Generated payload relocation, cleanup and bounded task prototypes | Task-close progress, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Implement owning panic outcomes and explicit synchronous failure propagation
   across scalar calls, following [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md).
   Prove diagnostic/effect preservation and original-cause cleanup in debug/release.
   Then add foundational identities, typed allocation failure and string-view origins
   before enabling bounded strings.Owned cleanup. Keep task-close progress,
   cancellation and pinned unwinding separate until their contracts are proved.
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
