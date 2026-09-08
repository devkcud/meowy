# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Shared-reference allocator carriers](compiler/ALLOCATOR_BOUNDS.md#shared-reference-allocator-carriers)
  preserve physical references and allocator lifetime bounds through whole replacement,
  field RHS effects, Leave and Restart. Current tags permit inactive nullable paths;
  active expired uses remain E303 and physical conflicts remain E302.
- Eight new library groups, one required-origin header check and four native groups
  cover this slice. All fourteen checks pass: 867 Rust (417 library, 450 native),
  35 Python, 54 debug/release examples, both editors, formatting, Clippy, build and
  contracts. Runtime passes all 100 groups/profile with sanitizers and required probes.
  Conformance: 10 passed, 13 unsupported, 0 failed. Runtime/backend unchanged.
- The [allocator-carriers example](compiler/examples/allocator-carriers.mwy) shows
  reference replacement while retaining lifetime-only allocator bounds. Lists,
  reference-only mutable carriers, mutable reference fields and emitted aliases remain
  gated. Dynamic contexts/views, owning cleanup, source error APIs, tasks, DWARF and
  the full release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Shared-reference allocator carriers, restart headers and panic outcomes | List/alias/reference bounds, dynamic origins and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend reference-only mutable carriers and mutable reference fields; emitted aliases need
   backing synchronization and lists need bounded element summaries. Preserve current
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
