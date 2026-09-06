# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `554fa6c` adds SlotAlias after mutable named initialization. Later reads,
  direct assignment and mixed SetPath writes use the actual result field cell;
  returned records observe the updates and ordinary copies remain independent.
  Initializers still execute once. Wider final slot types convert to/from the
  lexical alias type, with concrete union payload addresses for aggregate paths.
- Checker validation requires a compatible concrete mutable result field or proves
  the emission cannot contribute to completion. Discarded aliases retain valid
  initialized local cells for their remaining effects. Named outer targets, optional
  fields, own-target restart and inner-loop updates are covered.
- Alias identities are canonical per target field for write conflicts and predicate
  invalidation. Mutable facts seed unknown alias activity before origin analysis;
  stale initializer tags cannot erase loans, and unrelated reference fields keep
  their origins. Borrowing emitted storage remains B001.
- Coverage/example: `fda4a67` adds eight native groups and
  `compiler/examples/emitted-slots.mwy`. Nine library groups cover metadata budgets,
  mutable facts, cells, conversions, defaults, restart and discarded paths. Function
  return lowering now converts concrete bodies to their declared record unions.
- All 14 combined checks pass: 310 Rust tests, 35 Python tests, 860 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs emitted-slots with exact output. Independent checks
  and lifecycle executions pass. No active workers, unfinished code or failing
  checks remain. New helpers follow the existing modular source organization.
- Emitted borrows, shared-reference/temporary write roots, mutable reference-bearing
  fields, source-level exclusive references, owned cleanup and full release
  qualification remain open. Next is an explicit lifetime model for borrowed result
  storage, followed by runtime/module/library work.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Mutable result-slot aliases and unified checked write paths | Result-storage borrows, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Model borrowing emitted storage with explicit target-block/slot ownership rather
   than the alias name's lexical lifetime. Coordinate references, origin validation
   and canonical loan regions; test nested aliases, scope exit/restart and rejection
   of references escaping result publication. Keep B001 until storage lifetime is proved.
2. Preserve first-collection conflict rules and precise slot identity while adding
   capabilities. Shared-reference/temporary write roots, mutable reference-bearing
   fields and source-level exclusive references need explicit initialization and
   cleanup models before being enabled.
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
