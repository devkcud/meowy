# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Widened borrowed-alias backing](compiler/OWNERSHIP.md#widened-borrowed-alias-backing)
  supports writes to one exact result-union member. Shared origin/loan paths preserve
  outer tags, absent defaults, sibling versions, old copies and RHS/Leave effects.
- All fourteen combined checks pass: 943 Rust (462 library, 481 native), 35 Python,
  60 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 100 groups/profile with sanitizers and required probes. Conformance:
  10 passed, 13 unsupported, 0 failed. Runtime/backend and dependencies unchanged.
- The [widened-aliases example](compiler/examples/widened-aliases.mwy) updates an
  optional pointer and preserves the absent path. Surviving published headers, broader
  union views, lists, dynamic origins, owning cleanup, tasks and full release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Exact-member borrowed aliases and transient storage | List/alias/reference bounds, dynamic origins and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add surviving published result headers before lifting the ownership gate. Proper-subset union views and
   allocator-only bounds need separate proofs. Preserve outer tags, shared result
   paths, sibling versions, transient cell lifetime and RHS effects. Lists need
   bounded summaries; dynamic origins and [owning cleanup](compiler/OWNING_HIR.md) follow.
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
