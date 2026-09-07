# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `0c6492b` extends actual SlotAlias cells to immutable emitted references
  and reference-carrying records/unions. Existing component states retain pointee
  origins, input bounds and tag facts; copied references gain no dependency on the
  containing slot's lifetime.
- Concrete reference-free fields can be borrowed from locals, copied parameters/
  receivers and emitted carriers. A pure storage-path walk validates the selected
  type; crossing a stored reference still uses the existing pointee reborrow path.
  Selected field addresses follow their Local/Slot owner and report E303 on escape.
- Selected addresses do not read unrelated reference payloads. Pending result
  emissions still retain their contained references, preserving E302 conflicts.
  Outer/discarded target lifetimes, widening and restart use the existing storage
  machinery. Whole-carrier/reference-cell borrows remain B001.
- `888c67e` adds nine native groups, `compiler/examples/reference-slots.mwy`
  and README evidence. Four semantic and five backend groups distinguish physical
  cell ownership from stored references. Production changes fit two existing
  checker modules; backend lowering, runtime ABI and dependencies are unchanged.
- All 14 combined checks pass: 360 Rust tests, 35 Python tests, 863 local links,
  editors, schemas/catalog, formatting, Clippy, build and conformance. Runtime
  debug/release/sanitizer checks pass unchanged. Conformance remains 10 passed,
  13 unsupported, 0 failed in both profiles.
- The optimized compiler runs reference-slots with exact output. Twelve independent
  origin/lifetime/boundary cases and source review pass. No unfinished source work,
  active workers or failing checks remain; modular organization is preserved.
- Whole-carrier/reference-cell borrows need transitive pointee summaries before
  being enabled. Mutable reference carriers, exclusive references, temporary owners,
  owned cleanup, modules and full release qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Reference-bearing emitted slots and selected carrier-field borrows | Transitive carrier borrows, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Model transitive pointee summaries before whole-carrier/reference-cell borrows.
   Dereference copies must recover every contained origin and input bound separately
   from the borrowed cell's lifetime. Verify nested references, selected fields,
   copied inputs, widened/discarded slots and restart; keep B001 until proved.
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
