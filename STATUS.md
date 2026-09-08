# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Allocator return bounds](compiler/ALLOCATOR_BOUNDS.md) now follow immutable
  locals, records/unions, calls and shared reborrow snapshots. Escapes and expired
  consumption report E303. Active input lifetimes constrain even a returned heap
  handle; the old blanket signature gate is removed for supported result shapes.
- Lifetime-only allocator bounds do not create physical loans or forbid replacing
  still-live scalar storage. Actual references retain their access rules. Mutable,
  list and restart-header paths that cannot retain allocator bounds remain B001;
  existing unbounded static heap behavior remains supported.
- All fourteen combined checks pass: 819 Rust (385 library, 434 native), 35 Python,
  50 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 100 groups/profile under debug/release/ASan/UBSan/LSan plus required
  fatal/admission/guard/fiber probes. Runtime/backend representations did not change.
- Conformance remains 10 passed, 13 unsupported, 0 failed. The
  [allocator-bounds example](compiler/examples/allocator-bounds.mwy) exercises the
  new contract. Actual dynamic allocator contexts, owning string views/constructors,
  automatic cleanup, source error APIs, task recovery and full release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Nominal values, allocator return bounds and panic outcomes | Bound carriers, dynamic view/context origins and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend allocator bounds to mutable/list/header carriers without losing facts,
   then model actual dynamic allocator contexts and string-view origins in the same
   passes. Static heap is still the only allocator factory. Follow
   [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md) for bounded drop schedules before
   enabling strings.copy: normal/Leave/Restart/panic, retained/discarded emissions,
   temporary and call transfers must release exactly once. Add source failure APIs
   without implicit descriptor erasure; keep task cleanup/unwinding qualification separate.
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
