# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- [Exclusive restart authority](compiler/EXCLUSIVE_RESTARTS.md) now has a bounded
  implementation plan. The body-wide gate and opaque restart authority remain
  unchanged; shared descendants and demand-only headers need an explicit frontier
  proof before iteration-local exclusive carried-slot borrows can be enabled.

- [Shared carried-scalar borrowing](compiler/OWNERSHIP.md#shared-carried-scalar-borrows)
  passes source/proof and native debug/release checks. Acquisition
  requires active, initialized result storage; existing expiry and loan checks remain.
  Six new source groups, five native groups and the full compiler gate pass.

- [Carried scalar initialization](compiler/OWNERSHIP.md#carried-scalar-initialization)
  proves exactly-once emission across inner restarts for declared non-nullable
  scalar results, including Boolean flags, owner resets and completing paths.

- [Late published aliases](compiler/OWNERSHIP.md#late-published-aliases) initialize
  inside restarted bodies when their emission cannot reach any backedge. Explicit
  frontier certificates preserve empty headers, actual definitions and source lifetimes.

- [Changing published aliases](compiler/OWNERSHIP.md#changing-published-restart-results)
  now use canonical local headers and demand-only normal/Leave transfers. Binding
  guards, conditional backing, old copies and final source lifetimes are preserved.

- [Fixed published restart results](compiler/OWNERSHIP.md#fixed-published-restart-results)
  support borrowed alias writes before or after an inner restarted body. Stable
  snapshots retain backing types, conditional fields, old copies and source lifetimes.
  Changing preinitialized aliases and proven late initialization are supported;
  declared scalar initialization can also survive backedges after stateful proof.
- [Published result snapshots](compiler/OWNERSHIP.md#published-result-snapshots)
  retain backing types, tags, origins and bounds beyond alias lexical scopes.
  Fixed result bundles survive through checked identity; preinitialized changing
  aliases project canonical headers into the correct result backing domain.
- [Whole union-alias assignment](compiler/OWNERSHIP.md#whole-union-alias-assignment)
  maps lexical tags and reference paths into larger result unions. Old copies,
  branches/Leave, nested members and reset-iteration behavior remain intact.
- Current compiler gate: 1036 Rust (521 library, 515 native), 20 Python, 66 examples
  in both profiles, formatting, Clippy, build and contracts. Prior editor/runtime
  evidence is preserved: runtime 100 groups/profile with sanitizers and required
  probes. Conformance: 10 passed, 13 unsupported, 0 failed. Runtime/backend and
  dependencies are unchanged; runtime/editor checks were not rerun for this slice.
- The [union-aliases example](compiler/examples/union-aliases.mwy) changes a nullable
  pointer whose backing admits an extra string member. Wider carried initialization,
  union-view addresses/fields, lists, dynamic origins, owning cleanup and release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Whole union-alias assignment, late publications and shared carried-scalar borrows | Exclusive carried borrows, wider initialization, list bounds and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Static heap values, failure transport and private string payloads | Error APIs, owning source construction and module loading |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Implement the [exclusive restart authority plan](compiler/EXCLUSIVE_RESTARTS.md):
   prove no exclusive loan or descendant crosses a reset edge before narrowing
   opaque authority or either source gate. Keep exclusive headers unsupported;
   preserve indirect-write invalidation, move/last-use conflicts and owner expiry.
   Run source/native regressions and the compiler gate after implementation.
   Extend wider initialization/value summaries only after separate proof.
   Preserve late certificates,
   duplicate-slot checks and result-scope resets.
   Preserve conditional initialization,
   ancestor storage, exact predecessor coverage and no synthetic reads.
   Union-view addresses/field paths
   and allocator-only bounds need separate proofs. Preserve both tag domains, old
   copies, transient cell lifetime and RHS effects. Lists need bounded summaries;
   dynamic origins and [owning cleanup](compiler/OWNING_HIR.md) follow.
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
