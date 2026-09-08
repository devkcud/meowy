# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Fixed allocator record mutation](compiler/ALLOCATOR_BOUNDS.md#fixed-record-mutation)
  now preserves whole-value and pure-field versions, including nested records.
  Field commits use state after RHS effects, retain sibling writes and preserve
  earlier copies. Leave skips unfinished stores; restart expiry remains per component.
- Selected field reads check only the requested bounds. An expired allocator sibling
  does not block an independent scalar read or repair; whole-record consumption still
  requires all active bounds. Physical cell conflicts remain E302 and expired uses E303.
- All fourteen combined checks pass: 842 Rust (400 library, 442 native), 35 Python,
  52 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 100 groups/profile in debug/release/ASan/UBSan/LSan plus required
  fatal/admission/guard/fiber probes. Runtime/backend representations did not change.
- Conformance stays 10 passed, 13 unsupported, 0 failed. The
  [allocator-records example](compiler/examples/allocator-records.mwy) demonstrates
  selective read and repair. Bounded tagged/list/reference-bearing records and
  emitted aliases remain gated, as do owning constructors and automatic cleanup.
  Dynamic contexts/views, source error APIs, tasks, DWARF and full release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Mutable allocator records, restart headers and panic outcomes | Tagged/list/alias bounds, dynamic origins and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend bounded tagged/list/reference-bearing carriers and emitted aliases with
   their required variant, element and backing-storage proof before relaxing gates.
   Preserve pure field RHS effects and independent sibling versions. Then add actual
   dynamic allocator contexts and string-view origins; static heap is still the only
   factory. Follow [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md) for drop schedules
   before strings.copy: normal/Leave/Restart/panic and emitted/temporary/call transfers
   need exactly-once cleanup. Source error APIs and task qualification remain separate.
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
