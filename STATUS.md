# Meowy project status

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `4ddef85` adds forward leave-state merges for fixed shared-reference
  locals. Targets record the reference bindings that existed on entry; each leave
  captures those surviving values before inner scope restoration. Captured exits
  join normal fallthrough at the exact target before result completion is checked.
- Nested leaves preserve completed assignments and skip unfinished stores, calls,
  indices and short-circuit continuations. Capturing/joining adds no reference read.
  Old copies, physical cell loans, public bounds and target-local/slot/temporary
  expiry remain enforced. Reassignment combined with restart still reports B001.
- `b3e875a` fixes conditional emission proofs: disjoint result paths no longer
  cancel their proofs and hide conflicting writes. The previous compiler accepted
  the regression; current checks report E302. This fix is committed independently.
- `cb4afaf` adds eight native groups, `compiler/examples/leave-references.mwy`
  and README evidence. Four Leave-origin groups, one independent proof regression
  and six loan groups cover the implementation. `compiler/src/borrow/exits.rs` owns target
  snapshots; origin and loan joins reuse their bounded branch helpers.
- All ten compiler checks pass: 472 Rust tests, 20 Python tests, 869 local links,
  schemas/catalog, formatting, Clippy, build and conformance. All 33 examples run
  in debug/release. The optimized compiler runs leave-references with exact output;
  14 independent checks and six profile executions pass.
- Runtime/editor checks were not rerun; prior evidence is retained in the compiler
  tracker. HIR, backend storage, Facts shape, runtime ABI and dependencies are unchanged.
  Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- Next is bounded restart dataflow with initial/backedge versions, guard resets
  and iteration-local expiry. Existing budgets still apply, including persistent
  target/exit snapshots. Mutable reference carriers, exclusive/owned work, generated
  cleanup and full release qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Guarded and forward-leave shared-reference versions | Restart dataflow, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Design bounded restart dataflow in `compiler/src/borrow/` and
   `compiler/src/loans/`. Join initial and backedge reference versions at target
   headers, reset iteration-specific guards, and expire iteration-local/temporary
   sources without losing surviving owner/cell loans. Verify first/later iterations,
   conditional restart, old copies, call bounds and initialization before relaxing
   the restart gate; retain other unsupported storage/ownership forms explicitly.
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
