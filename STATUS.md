# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Nominal foundation values](compiler/FOUNDATION.md) now have checked HIR layouts.
  Static heap handles support ordinary copies, calls, storage and cell borrows;
  the [heap-handles example](compiler/examples/heap-handles.mwy) runs in both profiles.
  AllocationFailure facts retain nominal identity through calls, borrows and unions.
- Copyability, destruction and equality are separate. Opaque values cannot gain
  equality from aggregate storage or be forged by record construction. Source owner
  storage and backend values requiring cleanup schedules remain gated. Allocator
  returns from borrow-carrying inputs also remain B001 pending public return bounds.
- All fourteen combined checks pass: 808 Rust (377 library, 431 native), 35 Python,
  49 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 100 groups/profile in debug/release/ASan/UBSan/LSan plus required
  fatal/guard/admission/fiber probes. Both heap-handle ELFs import only libc.so.6.
- Conformance remains 10 passed, 13 unsupported, 0 failed. Source failure construction,
  metadata/erasure APIs, dynamic allocator/string-view origins and automatic resource
  cleanup remain open. Task recovery/cancellation, DWARF and full release are unqualified.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Nominal foundation layouts, static heap values and panic outcomes | Allocator/view origins and bounded drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add dynamic allocator/string-view origins and conservative allocator-return bounds
   to the existing borrow pipeline, following [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md).
   Build bounded initialized-state/drop schedules before enabling strings.copy:
   prove normal/Leave/Restart/panic cleanup with retained/discarded emissions,
   temporary and call transfers. Add source AllocationFailure construction/metadata
   without implicit descriptor erasure. Preserve the new static-heap, nominal-value
   and native allocation/release tests; keep task cleanup/unwinding qualification separate.
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
