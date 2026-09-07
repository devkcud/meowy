# Meowy project status

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current milestone

- Compiler: `b0c9756` assigns stable bounded restart-site identities. `1df163b`
  carries stored nullable/tagged union activity through restart headers, including
  nested variants and reference-bearing record pointees.
- Stable member choices preserve parent/child activation across analysis passes.
  Canonical origins and lifetime bounds follow their structural reference paths.
  Initial/restart-site snapshots prove predecessor activity before widening/reset;
  loan transfers use that source activation after future demand crosses the reset.
  Missing paths require proof of inactivity; active missing evidence remains B001.
- `74fac7c` adds eight native groups and `compiler/examples/header-activity.mwy`.
  Coverage includes null/full/null transitions, inactive final-use release, old
  copies, physical cells, nested variants, public bounds and once-only call effects.
  Tests and activity analysis live in focused modules.
- All ten compiler checks pass: 532 Rust tests, 20 Python tests, 872 local links,
  schemas/catalog, formatting, Clippy, build and conformance. All 36 examples run
  in debug/release. Optimized header-activity execution and independent twelve
  checks/six profile executions pass. One legacy B001 expectation was updated
  after all eight new native groups passed; the corrected selection passes.
- Existing replay/work/storage limits remain, including weighted choice seeds and
  predecessor snapshots. HIR and internal Facts metadata changed; generated storage,
  runtime ABI and dependencies did not. Runtime/editor checks were not rerun;
  historical evidence at `f16c30b` remains in the compiler handoff.
- Temporary and iteration-owned carried sources/bounds remain B001. Independent
  field/owner and temporal correlations may widen conservatively. Mutable reference
  carriers, exclusive/owned work, generated cleanup and release qualification remain
  open. Conformance is 10 passed, 13 unsupported, 0 failed in both profiles.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded nullable/tagged restart headers with predecessor proofs | Explicit expired-source identities, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Design explicit expired carried-source identities in `compiler/src/borrow_value/`,
   `compiler/src/borrow/restart.rs`, `compiler/src/borrow/replay.rs` and
   `compiler/src/loans/restarts.rs`. Target-owned Local/Slot and Temporary summaries
   must not revive when a static local, emitted slot or statement site runs again.
   Permit safe overwrite-before-use only after origin and loan proofs represent
   expiry. Exercise entry versus backedge expiry, nested targets, active variants,
   public bounds, old copies and reinitialization; retain B001 until supported.
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
