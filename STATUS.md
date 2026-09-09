# Meowy project status

Repository workflow: agents commit their completed, validated task changes by
coherent feature, fix, refactor or other concern, ordered by dependency, unless the
user requests otherwise. Unrelated changes stay outside those commits.

Updated: 2026-09-08. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
This file keeps the current handoff; prior work is available in Git history.
The full documented v0.0.1 release remains incomplete.

## Current milestone

- The agreed next run is the complete, bounded
  [documentation completion slice](COMPILER.md#documentation-completion-slice):
  attachment, checked links, doc check/build, basic API pages and checked examples,
  including validation and commits. No further preparatory milestones or polishing
  detours; full LSP/rename and HTTP/TLS implementation stay separate.
- STEP logs are retired. Plans live in COMPILER.md and current handoffs in STATUS;
  this planning/cleanup change does not rerun the compiler or editor gates below.
  Local validation passes: 1070 links in 101 Markdown files and Git whitespace checks.

- Documentation-fence lexing passes all eleven focused tests; Vim/Neovim suites
  also pass after the approved fixture correction. Tokens retain declaration/module
  kind, bar count and exact spans;
  unclosed fences use E002. Compilation keeps attachment/checking explicitly B001.
  All ten compiler checks pass, including three new native regression groups.

- [HTTP](docs/reference/stdlib/http.md), [TLS](docs/reference/stdlib/tls.md) and
  [checked documentation comments](docs/reference/documentation.md) are now specified.
  HTTP separates messages, clients, streaming services and typed route policies;
  docs are specified to attach to syntax and reuse compiler types/links/examples.
  HTTP/TLS implementation, protocol qualification and semantic documentation tooling
  remain unavailable. This slice implements lexical recognition and editor regions.
- Compiler/native and editor checks ran for this slice. Native runtime/sanitizer
  evidence was not rerun; no HTTP/TLS or executable documentation-example support
  is implied by those checks.

- [Mixed shared-header precision](compiler/EXCLUSIVE_RESTARTS.md#certified-shared-headers)
  passes source/graph and native checks. Header metadata
  retains per-definition active-path obligations; every predecessor must cover
  them before header-only opacity can be excluded from the exclusive frontier proof.
  Genuine unknown ancestry and live exclusive descendants remain gated.
  All ten compiler checks pass, including the new mixed-headers example in both profiles.

- [Local exclusive carried-scalar borrows](compiler/EXCLUSIVE_RESTARTS.md) pass the
  compiler/native gate. A conservative ancestry proof follows copies, header
  transfers and parents; live exclusive or opaque demand rejects a backedge.
  Only certified carried-scalar loans bypass blanket restart opacity.

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
- Current compiler gate: 1080 Rust (552 library, 528 native), 20 Python, 68 examples
  in both profiles, formatting, Clippy, build and contracts. Both editor suites pass.
  Prior runtime evidence remains 100 groups/profile with sanitizers and required
  probes, not rerun here. Conformance: 10 passed, 13 unsupported, 0 failed.
  Runtime/backend and dependencies are unchanged.
- The [union-aliases example](compiler/examples/union-aliases.mwy) changes a nullable
  pointer whose backing admits an extra string member. Wider carried initialization,
  union-view addresses/fields, lists, dynamic origins, owning cleanup and release remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Whole union-alias assignment, carried scalars and certified mixed shared headers | Reference-free record initialization, list bounds and drop schedules |
| Runtime | Private owned strings, generated cleanup and bounded task prototypes | Source ownership integration, task close and cancellation |
| Standard library | Existing storage/error work; HTTP/TLS contracts now specified, not implemented | Module/ownership prerequisites, protocol fixtures and TLS provider qualification |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim doc-fence highlighting and existing regressions pass | Shared documentation model, structural attachment and LSP integration |
| Developer tools | Combined verification exists; doc commands are specified, not implemented | Checked doc attachment/links/examples and safe local publication |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Complete the [one-run documentation slice](COMPILER.md#documentation-completion-slice)
   end to end, including attachment diagnostics, compiler-bound links, doc commands,
   basic API pages, checked/explicitly run examples, validation and cohesive commits.
   Defer full LSP/rename integration and optional rendering work, then return to
   core compiler features rather than adding another documentation milestone.
2. Resume the compiler's declared reference-free record-slot initialization across inner restarts,
   using bounded shapes and full-slot availability before widening scalar-only
   eligibility. Keep nullable/union/reference-bearing carried initialization gated;
   qualify storage borrowing and exclusive projections separately. Preserve certified
   shared-header coverage, call/input opacity and the exclusive backedge boundary;
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
3. Plan HTTP/TLS implementation against source ownership, module and runtime support:
   parser/serializer fixtures first, explicit memory/transport limits, qualified TLS,
   then clients/services and closed route policies. Protocol execution needs its own
   evidence; the new chapters are contracts, not completed libraries.
4. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
5. Extend aggregate/emitted-name/cross-element constraints in the list-context
   modules without replay or stale facts. Build the manifest/module graph for Meowy
   libraries and documented projects; keep runtime/editor/library progress visible.
6. Preserve current compiler evidence; do not rerun green checks without a new change
   or concern. Run `python3 -B tools/verify.py --all` after wider integrations. LSan
   needs process inspection. Strict conformance still has 13 unsupported cases;
   this bootstrap gate and host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
