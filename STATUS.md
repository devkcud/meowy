# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Scalar panic outcomes](compiler/PANIC_OUTCOMES.md) now propagate owning snapshots
  across generated calls. Results are written only on success; failed calls skip
  remaining arguments and caller effects. Nested/abandoned panic messages preserve
  the existing streamed diagnostic bytes and original failure.
- Runtime snapshots support bounded streamed capture (`ef935da`). Five new native
  LLVM groups prove returned P001/P002/P003/P006 cleanup, original-cause P008,
  independent copied evidence, UTF-8 truncation and result publication. Source
  recursion and all existing scalar/reference/list behavior pass in debug/release.
- All fourteen combined checks pass: 795 Rust (366 library, 429 native), 35 Python,
  48 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 93 groups per debug/release/sanitized profile plus required fatal,
  guard, admission and fiber probes. A generated outcome ELF imports only libc.so.6.
- Conformance remains 10 passed, 13 unsupported, 0 failed. Automatic resource cleanup,
  source recovery, cancellation, DWARF and full release qualification remain open.
  The [owning-HIR design](compiler/OWNING_HIR.md) keeps strings.Owned gated until its
  foundational types, allocation failures, origins and drop schedules are implemented.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Scalar ownership, generated bridge and explicit panic outcomes | Foundational owner types and bounded cleanup schedules |
| Runtime | Generated payload relocation, cleanup and bounded task prototypes | Task-close progress, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Establish resolved foundational item/type identities for memory/strings, typed
   allocation failure and the static heap allocator contract, following
   [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md). Prove constructor failure and
   exactly-once release in the private ABI before enabling strings.Owned source.
   Add dynamic string-view origins and bounded ownership/drop schedules, retaining
   the new synchronous panic outcome tests. Keep task close/cancellation and pinned
   unwinding separate until their contracts are proved.
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
