# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Alias Restart ownership](compiler/OWNERSHIP.md#restart-ownership-for-alias-writes)
  permits borrowed alias writes when Restart resets their slots or is independent.
  Written outer result owners enclosing an inner Restart stay gated. Named emission
  targets, current tags, RHS/Leave effects and expired slot-copy checks are preserved.
- All fourteen combined checks pass: 918 Rust (448 library, 470 native), 35 Python,
  58 debug/release examples, both editors, formatting, Clippy, build and contracts.
  Runtime passes 100 groups/profile with sanitizers and required probes. Conformance:
  10 passed, 13 unsupported, 0 failed. Runtime/backend unchanged.
- The [alias-restarts example](compiler/examples/alias-restarts.mwy) initializes fresh
  slots and publishes the last iteration's updated pointer. Surviving result headers,
  broader backing, lists, dynamic origins, owning cleanup, tasks and full release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Borrowed alias writes in restarted scopes | List/alias/reference bounds, dynamic origins and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Add surviving outer result-header merging before lifting the ownership gate. Widened/discarded
   backing and allocator-only aliases need separate proofs. Preserve emission-target
   ancestry, fresh result definitions, expired-source checks and RHS effects. Lists
   need bounded summaries; dynamic origins and [owning cleanup](compiler/OWNING_HIR.md)
   follow those proofs.
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
