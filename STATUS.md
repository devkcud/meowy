# Meowy project status

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `89800dc` adds bounded restart analysis for mutable references to
  reference-free pointees. Initial and feasible backedge source/bound sets grow
  to a fixed point before the final facts publish. Fresh body passes discard
  provisional facts and exit queues while sharing charged work and guard storage.
- Canonical headers use direct source sets with guards reset to TRUE. Stable loan
  header IDs receive demand-only transfers from initial/backedge predecessors;
  old copies and pre-loop source precision stay separate. Header entry/reset
  deliberately loses branch correlations, so some safe programs may be rejected.
- In bodies using reference reassignment, every mutable reference at restarted
  target entry must have reference-free T and surviving ancestor Local/Slot/Input
  sources and bounds. Temporary and target/descendant-owned header sources remain
  B001; reference-bearing carried pointees need further work. Iteration-local
  rebindings that do not enter such headers retain existing support.
- `ca037d3` adds eleven native groups, `compiler/examples/restart-references.mwy`
  and README evidence. Four origin and six loan groups cover first/later values,
  nested targets, old copies, cell/public bounds, ref-free aggregates and source
  limits. Replay and header work live in focused origin/loan modules.
- All ten compiler checks pass: 493 Rust tests, 20 Python tests, 870 local links,
  schemas/catalog, formatting, Clippy, build and conformance. All 34 examples run
  in debug/release. The optimized compiler runs restart-references with exact output;
  twelve independent checks and six profile executions pass.
- Replay has a 64-pass cap plus existing shared work, 4,096-part and 262,144 weighted
  fact/cache limits. Retained/cloned header seeds and prior body facts are counted;
  this is a logical budget, not a byte-accurate allocator peak. Facts gains internal
  header metadata; HIR, backend storage, runtime ABI and dependencies are unchanged.
- Runtime/editor checks were not rerun; historical evidence remains in the compiler
  tracker. Conformance is still 10 passed, 13 unsupported, 0 failed in both profiles.
  Next is canonical transitive header components, followed by guarded activity and
  expired-iteration identities. Exclusive/owned work and release qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Bounded restart headers for surviving direct references | Transitive/expired-source headers, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend `compiler/src/borrow/restart.rs` and `compiler/src/loans/restarts.rs`
   to canonical per-component sources/bounds, starting with non-union reference-bearing
   pointees. Preserve Deref paths and lazy contents without inventing active variants
   or losing call/cell bounds. Prove convergence before enabling transitive headers;
   model guarded activity separately and retain Temporary/iteration-owned gates until
   explicit expired identities prevent same-site storage revival.
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
