# Meowy project status

Updated: 2026-09-07. This is the restart point for work across the repository.
The compiler has its detailed handoff in [compiler/STATUS.md](compiler/STATUS.md).
Historical checkpoints are in [STATUS_STEP_LOG.md](STATUS_STEP_LOG.md).
The full documented v0.0.1 release remains incomplete.

## Current snapshot

- Compiler: `34d2e7b` extends restart headers to nested references and record
  pointees that need no stored variant activity. One charged Shape validator is
  shared by origin and loan analysis, checks typed reference paths and requires
  actual-origin coverage for every path; lifetime bounds cannot replace pointers.
- Canonical convergence and deduplication use `(Path, Source)` separately for
  actual origins and bounds. Deref/Slot components stay distinct even when owners
  repeat in different fields or layers. Existing predecessor transfers remain
  demand-only, preserving lazy pointee reads, old copies and physical cell loans.
- Shape traversal stops at reference-free referents, preserving scalar-union
  references even through another reference or a carrier field. A union reached
  inside a stored summary still requires activity and reports B001. Temporary and
  iteration-owned sources/bounds remain B001 at every nested header component.
- `fa70eab` adds eight native groups, `compiler/examples/transitive-restarts.mwy`
  and README evidence. Four origin and six loan groups cover field/path identity,
  full coverage, lazy versus demanded reads, public bounds, nested targets and
  source/activity boundaries. Shared shape code lives in `compiler/src/borrow/header.rs`.
- All ten compiler checks pass: 511 Rust tests, 20 Python tests, 871 local links,
  schemas/catalog, formatting, Clippy, build and conformance. All 35 examples run
  in debug/release. The optimized example and twelve independent checks/six
  profile executions pass. One test expectation was corrected to retain an earlier E303.
- Existing 64-pass, shared-work, 4,096-part and weighted fact/cache caps remain.
  Runtime/editor checks were not rerun; their historical evidence is retained.
  HIR, Facts shape, backend storage, runtime ABI and dependencies are unchanged.
  Conformance remains 10 passed, 13 unsupported, 0 failed in both profiles.
- Next is canonical guarded header activity with stable convergence and proved
  inactive-path transfers. Expired iteration identities, mutable reference carriers,
  exclusive/owned work, generated cleanup and release qualification remain open.

## Still to build or qualify

| Area | Current boundary | Next useful work |
| --- | --- | --- |
| Compiler | Typed transitive restart headers without stored activity | Guarded activity/expired-source headers, exclusive ownership and cleanup |
| Runtime | Owning panic snapshots and failure batches | Generated scope exits, richer diagnostics, cancellation and DWARF |
| Standard library | Foundational compiler intrinsics only | Concrete module loading and first Meowy library layer |
| Packages | Manifests detected but unsupported by bootstrap | Typed manifest model and module graph |
| Editor | Vim/Neovim files and regression checks exist | Shared analysis service, then LSP integration |
| Developer tools | Combined repository/editor/compiler/runtime verification works | CI/bootstrap environment and complete library/tool coverage |
| Distribution | Host-only native bootstrap | Bundled sysroot, reproducibility and minimum-host qualification |

## Next steps

1. Extend `compiler/src/borrow/header.rs`, `compiler/src/borrow/restart.rs` and
   `compiler/src/loans/restarts.rs` with canonical active-member alternatives and
   stable guard identities. Preserve correlations between variants and reference
   components; inactive paths must be proved absent before omitting transfers.
   Validate initial/backedge null/ref transitions, copies, tag-only reads and public
   bounds before relaxing activity B001. Keep Temporary/iteration-owned source gates
   until explicit expired identities prevent same-site revival.
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
