# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Mutable borrowed carriers](compiler/OWNERSHIP.md#mutable-borrowed-carriers) now
  include reference-only records and nullable/closed unions. Whole replacement,
  field RHS effects, current predicates, Leave and Restart preserve component
  origins and loans. Active expired uses stay E303 and physical conflicts E302.
- Nine new library tests and five native groups pass, including debug/release
  execution and missing-origin header proof. All fourteen check categories have
  passing evidence: 881 Rust (426 library, 455 native), 35 Python, 55 examples,
  both editors, formatting, Clippy, build, contracts and runtime sanitizers.
  Conformance: 10 passed, 13 unsupported, 0 failed. Runtime/backend unchanged.
- The [mutable-carriers example](compiler/examples/mutable-carriers.mwy) alternates
  empty/live/empty reference fields through replacement and Restart. Mutable
  reference fields, emitted aliases, lists and exclusive carriers remain gated.
  Dynamic contexts/views, owning cleanup, source error APIs, tasks, DWARF and the
  full release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Fixed mutable borrowed carriers, restart headers and panic outcomes | List/alias/reference bounds, dynamic origins and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add mutable reference fields and emitted aliases with constructor/backing
   synchronization and lists need bounded element summaries. Preserve current
   tag observations, post-RHS sibling versions and active-reference header coverage.
   Then add dynamic allocator contexts and string-view origins. Follow
   [compiler/OWNING_HIR.md](compiler/OWNING_HIR.md) before strings.copy: normal,
   Leave, Restart, panic and emitted/temporary/call transfers need exactly-once cleanup.
   Source error APIs and task qualification remain separate.
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
