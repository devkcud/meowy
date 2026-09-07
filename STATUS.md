# Meowy project status

Updated: 2026-09-06. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `5c55e44` adds guarded assignment merges for fixed shared-reference
  locals in matcher arms and short-circuit right operands. Each arm starts from
  the values present after condition effects; only returning paths reach the join.
  Skipped assignments preserve incoming values, and earlier copies stay fixed.
- Origin/bound/activity proofs are masked by returning guards. Loan merges transfer
  demand on each predecessor without adding reads; unchanged versions are reused
  and duplicate origins are normalized. Physical reference cells keep their identity,
  so guarded cell views and selected-owner writes retain E302 protection.
- `Facts.merging` activates the new path per body. Assignment-free bodies retain
  their existing loop behavior. Bodies combining reference reassignment with any
  leave/restart remain B001 until exit states and loop fixed points are implemented.
- `af3a868` adds eight native groups, `compiler/examples/guarded-references.mwy`
  and README evidence. Four origin and six loan groups cover partial assignments,
  nested guards, temporary/scoped expiry, public bounds, nullable pointees and panic.
  Focused helpers keep the new analysis in `compiler/src/borrow/branches.rs` and
  `compiler/src/loans/branches.rs`.
- All ten compiler checks pass: 453 Rust tests, 20 Python tests, 868 local links,
  schemas/catalog, formatting, Clippy, build and conformance. All 32 examples execute
  in debug/release. The optimized compiler runs guarded-references with exact output;
  12 independent checks and six profile executions pass.
- Runtime/editor checks were not rerun this slice; their earlier evidence is retained
  in the compiler handoff. HIR, backend storage, runtime ABI and dependencies are
  unchanged. Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- Branch work stays bounded: 64 repeated joins pass; 1,024 exceed the charged loan
  work budget with B001. Next is forward leave-state merging, followed by restart
  fixed points. Mutable reference carriers, exclusive/owned work and release
  qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded shared-reference versions and bounded transitive borrowing | Leave/restart state flow, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Model forward leave-state snapshots in `compiler/src/borrow/` and
   `compiler/src/loans/`. Capture surviving reference versions at each leave edge,
   merge them with fallthrough at the named target, and verify skipped RHS stores,
   old copies, cell loans and target-owned/temporary lifetimes. Relax the leave gate
   only after these paths are proved; keep restart B001 until a bounded fixed point
   accounts for incoming and loop-carried versions with reset guards.
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
