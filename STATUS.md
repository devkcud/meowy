# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `d3e6b12` unifies field/list assignment as SetPath with ordered Field
  and Index steps. Paths such as `holder.items[i].value` and `rows[i].items[j]`
  work on ordinary mutable reference-free Copy locals. Every crossed field must
  be mutable, and each selected list supplies its own initialized length.
- Indices are captured and bounds-checked once from root to leaf before later
  effects; P001 identifies the failing indexed prefix. Only the selected payload
  is stored. Bounds failure and nonreturning operands skip every later phase.
- Static fields before the first index identify the protected whole-collection
  region. Holder siblings outside it can be disjoint; retained views anywhere
  within it conservatively conflict. Each returning index and the final store use
  the reservation. Pure-field paths preserve same-type RHS owner replacement.
- Coverage/example: `4f6f2d2` adds seven native groups and
  `compiler/examples/mixed-writes.mwy`. Six library groups cover typed paths,
  regions, layouts, bounds, captured indices and early exits. Existing field/list
  write fixtures use the shared representation; obsolete boundary rows are updated.
- All 14 combined checks pass: 293 Rust tests, 35 Python tests, 859 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs mixed-writes with exact output. Twelve independent
  review cases pass. No active workers, unfinished code or failing checks remain.
  Modular source organization and the runtime ABI are preserved.
- Shared-reference/temporary/emitted roots, mutable primary slots, mutable
  reference-bearing fields, source-level exclusive references and owned cleanup
  remain outside this bootstrap. Next is real result-slot aliasing for mutable
  emitted names, followed by runtime/module/library and release work.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Unified mixed checked write paths and modular analyses | Result-slot aliases, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Give mutable emitted names real result-slot aliases before permitting their
   assignment. Model construction order, enclosing labels, leave/restart and result
   publication in HIR/checker/backend/loans; verify read-after-write inside the
   block and the final returned field, not just a detached local copy.
2. Preserve or improve region precision only with explicit alias/lifetime proof.
   Keep shared-reference/temporary roots and reference-bearing mutation closed
   until source-level exclusive references, initialization and cleanup are modeled.
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
