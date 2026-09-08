# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Mutable allocator bounds](compiler/ALLOCATOR_BOUNDS.md#mutable-versions-and-restart)
  now survive direct binding assignment, branches, short circuits, Leave and Restart.
  Earlier copies retain their own constraints; overwriting an expired handle before
  reading it is allowed. Active expired reads remain E303, and cell-borrow conflicts E302.
- Headers preserve explicit empty allocator versions and optional bound paths,
  including nullable allocator components beneath shared references. Physical
  reference paths still require complete origin proof. Ended Local/Temporary/Slot
  sources stay expired even when the same source site initializes again.
- All fourteen combined checks pass: 832 Rust (394 library, 438 native), 35 Python,
  51 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 100 groups/profile in debug/release/ASan/UBSan/LSan plus required
  fatal/admission/guard/fiber probes. Runtime/backend representations did not change.
- Conformance remains 10 passed, 13 unsupported, 0 failed. The
  [mutable-allocators example](compiler/examples/mutable-allocators.mwy) proves the
  new loop behavior. Bounded aggregate/list/emitted-alias mutation, actual dynamic
  allocator contexts, owning string views/constructors and automatic cleanup remain open.
  Source error APIs, task recovery/cancellation, DWARF and full release remain unqualified.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Mutable allocator bounds, restart headers and panic outcomes | Aggregate/list bounds, dynamic origins and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend bounded allocator record/union/list carriers and emitted aliases, preserving
   field/element versions and tag activity before removing their B001 guards.
   Then add actual dynamic allocator contexts and string-view origins; static heap
   remains the only factory. Follow [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md)
   for bounded drop schedules before strings.copy: normal/Leave/Restart/panic and
   emitted/temporary/call transfers must release exactly once. Source failure APIs
   must preserve explicit erasure; task cleanup/unwinding remains separately qualified.
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
