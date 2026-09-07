# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `dd28a65` enables shared borrows of reference-free Copy temporary owners.
  Statement wrappers track lifetime without changing ordinary lexical bindings;
  TemporaryBorrow evaluates once into a dedicated typed cell. Source::Temporary
  retains the materialization site and owning statement through calls/reborrows.
- Matcher conditions share their directly controlled statement's lifetime; nested
  block statements get separate owners. Later reference use is E303, even for
  literals. Same-statement calls, dispatch, field/index paths and value copies work;
  leave/restart/panic skip nonreturning materialization and later effects.
- Function entry now validates all active transitive argument origins and bounds
  after arguments return. This closes an expired temporary hidden behind a live
  reference cell in a scalar-returning call. Tag-only origin inspection avoids
  payload reads while still validating holder pointers and computed effects.
- `83987ec` adds nine native groups, `compiler/examples/temporary-borrows.mwy`
  and README evidence. Six origin, two loan and four backend groups cover lifetime,
  identity, effects, bounds and the two cross-feature fixes. Runtime ABI/dependencies
  are unchanged; Copy values add no owned cleanup or loop stack growth.
- All 14 combined checks pass: 402 Rust tests, 35 Python tests, 865 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs temporary-borrows with exact output. Fifteen independent
  checks and six profile executions pass; no unfinished source work, active workers
  or failing checks remain. Existing operand widths are preserved; borrowing does
  not perform reference-width conversions.
- Reference-bearing/owned temporary owners, exclusive references, mutable reference
  carriers, reference-bearing lists and full release qualification remain open.
  Next is Copy temporary contents carrying references, with transitive summaries
  kept separate from the temporary cell's statement lifetime.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Statement-owned reference-free Copy temporaries and nested shared borrows | Reference-bearing Copy temporaries, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend temporary Copy owners to values carrying references using stored-value
   summaries beneath Deref. Preserve contained origins, bounds and tags independently
   of cell lifetime; direct copies may keep surviving pointees while temporary-cell
   addresses still expire. Verify call bounds, nested statements, effects and budgets.
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
