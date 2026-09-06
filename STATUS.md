# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `6ff8807` preserves named field mutability as HIR Field metadata and
  implements checked field assignment on ordinary mutable reference-free Copy
  locals. Every crossed field must be mutable; E305 rejects immutable paths.
  Field flags participate in type identity, constructor matching and union tags
  while physical field layout remains unchanged.
- SetField evaluates RHS once and stores only the selected field. Static offsets
  permit same-shape Copy owner replacement during RHS; new sibling values survive.
  Disjoint shared views and final-use reads are allowed, while overlapping live
  views fail E302. Written/ancestor/descendant facts are invalidated with charged
  work; disjoint sibling facts remain valid.
- Construction/forwarding/list contexts preserve field flags. Expected or branch
  mutability conflicts report E206; whole-record type mismatch remains E207.
  Emitted-name dependencies in unresolved shape probes stay Unknown/B001 instead
  of accidentally resolving an outer binding.
- Coverage/example: `ab813f8` adds seven native groups and
  `compiler/examples/mutable-fields.mwy`. Seven library groups cover field shapes,
  access rules, loans, unchanged layouts, selected stores and early exits.
- All 14 combined checks pass: 280 Rust tests, 35 Python tests, 858 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs mutable-fields with exact output. Fourteen independent
  review cases pass, including inherited input bounds. No active workers, unfinished
  code or failing checks remain. Modular source organization is retained.
- Mixed field/index writes, shared-reference/temporary targets, mutable primary
  slots, mutable reference-bearing fields and direct emitted-name assignment remain
  B001. Owned cleanup, source-level exclusive references, modules and full release
  qualification are still open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Mutable record shapes, checked field/list writes and modular analyses | Mixed checked paths, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend checked write paths to mix mutable named fields and initialized indices
   (`holder.items[i]`, `items[i].field`). Coordinate HIR, `check/mutation.rs`, list
   checking, loans and backend pointer traversal; preserve field gates, per-index
   bounds order, parent storage and precise/conservative overlap as appropriate.
2. Give mutable emitted names real result-slot aliases before permitting assignment
   to them. Preserve construction order, named exits/restarts and returned storage;
   do not mutate a detached local copy. Keep reference-bearing fields and source
   exclusive references closed until their origin/initialization rules are modeled.
3. Define generated payload/diagnostic layouts and connect cleanup to runtime
   mark/close while parent storage lives. Retain owning outcomes, drain every failure
   batch and preserve interleaved cleanup before cancellation and unwinding.
4. Add richer source identities and diagnostic evidence/artifacts; current bounded
   snapshots and byte-span text do not implement complete release replay.
5. Extend aggregate/emitted-name/cross-element constraints in the new list-context
   modules without replay or stale facts. Build the manifest/module graph for Meowy
   libraries and documented projects; keep runtime/editor/library progress visible.
6. Run `python3 -B tools/verify.py --all` after integrations. LSan needs process
   inspection. Strict conformance still has 13 unsupported cases; the bootstrap
   gate and this host do not qualify a complete v0.0.1 release.

Preserve the unchanged vendored `fcontext.hpp` trailing blank line: its SHA-256
matches upstream. Its historical blank-at-EOF exception does not apply to project
files, which pass normal Git whitespace checks.
