# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Foundational identities](compiler/FOUNDATION.md) resolve memory/strings modules,
  heap/copy items and Allocator/AllocationFailure/Owned types through lexical aliases.
  Ordinary shadowing remains intact. Partial module APIs and runtime resource use
  stay B001 until modeled; no source constructor or automatic cleanup was enabled.
- The [private owned-string runtime](runtime/STRINGS.md) reserves Owned metadata,
  copies into explicitly selected heap storage and commits only on success.
  Empty values allocate nothing; typed exhaustion leaves an empty owner for retry.
  Existing descriptors and transfer/cleanup frames provide exactly-once release.
- All fourteen combined checks pass: 800 Rust (370 library, 430 native), 35 Python,
  48 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 100 groups/profile in debug/release/ASan/UBSan/LSan plus required
  fatal, guard, admission and fiber probes. Generated heap-string ELF imports only libc.so.6.
- Conformance remains 10 passed, 13 unsupported, 0 failed. Native failure evidence
  and gated type identities are not lowerable source error/resource types. Next:
  source nominal layouts, dynamic view/allocator origins and bounded drop schedules.
  Source recovery, cancellation, DWARF and full release qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Foundation identities, scalar ownership and panic outcomes | Lowerable owner/error types, origins and cleanup schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Gated memory/strings identities and private string payloads | Source resource/error storage and concrete module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend the foundation identities to lowerable nominal resource/error types and
   string-view/allocator origins, following [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md).
   Keep AllocationFailure concrete and non-descriptor-compatible. Build bounded
   initialized-state/drop schedules before enabling strings.copy: prove normal,
   Leave, Restart and panic cleanup with retained/discarded emissions and temporaries.
   Preserve the new native allocation/release and alias/shadowing tests. Task close,
   cancellation and pinned unwinding remain separately qualified work.
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
