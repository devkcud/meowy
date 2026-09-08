# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Borrowed emitted-alias writes](compiler/OWNERSHIP.md#borrowed-emitted-alias-writes)
  now synchronize fixed reference-bearing names with exact result backing. Whole and
  field writes preserve sibling versions, old copies, current tags and RHS/Leave effects.
- All fourteen combined checks pass: 906 Rust (441 library, 465 native), 35 Python,
  57 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 100 groups/profile with sanitizers and required probes. Conformance:
  10 passed, 13 unsupported, 0 failed. Runtime/backend and dependencies unchanged.
- The [alias-writes example](compiler/examples/alias-writes.mwy) replaces an emitted
  pointer while retaining an old copy. Restart with these writes, widened/discarded
  backing and bounded allocator-only aliases remain gated. Lists, exclusive carriers, dynamic
  origins, owning cleanup, source error APIs, tasks, DWARF and full release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Exact-backing borrowed alias writes and mutable reference fields | List/alias/reference bounds, dynamic origins and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend result snapshots into restart merging before lifting the body-wide Restart gate. Add widened/discarded
   backing and allocator-only alias proofs, preserving selected result definitions,
   old copies and RHS sibling effects. Lists need bounded summaries. Add dynamic
   allocator/view origins, then follow [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md)
   for normal/Leave/Restart/panic and temporary/result cleanup transfers.
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
